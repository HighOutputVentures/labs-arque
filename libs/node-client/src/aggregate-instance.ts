import { Client } from './client';
import { Command, CommandHandler, GeneratedEvent } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';
import backoff, { Backoff } from 'backoff';
import { InvalidAggregateVersionError } from './error';
import R, { last } from 'ramda';
export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;
  private eventHandlers: Map<number, EventHandler<TState, TContext>> =
    new Map();

  private backoffInstance: Backoff;

  private commandHandlers: Map<
    number,
    CommandHandler<TCommand, TEvent, TState, TContext>
  > = new Map();

  constructor(
    private _id: ObjectId,
    private _version: number,
    private _state: TState,
    private client: Client,
    commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[],
    eventHandlers: EventHandler<TState, TContext>[]
  ) {
    this.mutex = new Mutex();

    for (const commandHandler of commandHandlers) {
      this.commandHandlers.set(commandHandler.type, commandHandler);
    }

    for (const eventHandler of eventHandlers) {
      this.eventHandlers.set(eventHandler.type, eventHandler);
    }

    this.backoffInstance = backoff.fibonacci({
      randomisationFactor: 0,
      initialDelay: 10,
      maxDelay: 300,
    });

    this.backoffInstance.failAfter(10);
  }

  get id() {
    return this._id;
  }

  get version() {
    return this._version;
  }

  get state() {
    return this._state;
  }

  private async digest(events: TEvent[]) {
    for (const event of events) {
      const eventHandler = this.eventHandlers.get(event.type);

      assert(eventHandler, `event handler for ${event.type} not found`);

      const state = await eventHandler.handle(
        {
          state: this._state,
        } as TContext & { state: TState },
        event
      );

      this._state = state;
      this._version = event.aggregate.version;
    }
  }

  public async process(command: TCommand): Promise<void> {
    const context = this;
    this.backoffInstance.on('ready', async function () {
      try {
        await context.processCommand(command);
      } catch (err) {
        if (err instanceof InvalidAggregateVersionError)
          context.backoffInstance.backoff();

        throw err;
      }
    });

    this.backoffInstance.backoff();
  }

  private async processCommand(command: TCommand): Promise<void> {
    const release = await this.mutex.acquire();

    try {
      let events = await this.client.listAggregateEvents({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(events as TEvent[]);

      const commandHandler = this.commandHandlers.get(command.type);

      assert(commandHandler, `command handler for ${command.type} not found`);

      const generatedEvent = await commandHandler.handle(
        {
          state: this._state,
        } as TContext & { state: TState },
        command
      );

      let lastEvent = null;

      if (Array.isArray(generatedEvent)) {
        let i = 0;
        let len = generatedEvent.length;
        for await (const event of generatedEvent) {
          const eventData = {
            ...event,
            aggregate: {
              id: this._id,
              version: this._version + i,
            },
            meta: {},
          };
          await this.client.insertEvent(eventData);

          if (len === i + 1) lastEvent = eventData;

          i += 1;
        }
      } else {
        const eventData = {
          ...generatedEvent,
          aggregate: {
            id: this._id,
            version: this._version + 1,
          },
          meta: {},
        };
        await this.client.insertEvent(eventData);

        lastEvent = eventData;
      }

      await this.digest([lastEvent]);

      release();
    } catch (err) {
      release();

      throw err;
    }
  }

  public async reload(): Promise<void> {
    const release = await this.mutex.acquire();

    try {
      let events = await this.client.listAggregateEvents({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(events as TEvent[]);

      release();
    } catch (err) {
      release();

      throw err;
    }
  }
}
