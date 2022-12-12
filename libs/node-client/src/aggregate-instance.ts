import { Client } from './client';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';
import backoff, { Backoff } from 'backoff';
import { InvalidAggregateVersionError } from './error';

export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;
  private eventHandlers: Map<number, EventHandler<TState, TContext>> =
    new Map();

  private backoff: Backoff;

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

    this.backoff = backoff.fibonacci({
      randomisationFactor: 0,
      initialDelay: 10,
      maxDelay: 300,
    });

    this.backoff.failAfter(10);
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

  public async process(command: TCommand): Promise<void> {
    const context = this;
    this.backoff.on('ready', async function () {
      try {
        await context.processCommand(command);
      } catch (err) {
        if (err instanceof InvalidAggregateVersionError)
          context.backoff.backoff();
      }
    });

    try {
      await this.processCommand(command);
    } catch (err) {
      if (err instanceof InvalidAggregateVersionError)
        context.backoff.backoff();

      throw err;
    }
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

      for (const event of events) {
        const eventHandler = this.eventHandlers.get(event.type);

        assert(eventHandler, `event handler for ${event.type} not found`);

        const state = await eventHandler.handle(
          { state: this._state } as never,
          event
        );

        this._state = state;
        this._version = event.aggregate.version;
      }

      const commandHandler = this.commandHandlers.get(command.type);

      assert(commandHandler, `command handler for ${command.type} not found`);

      const generatedEvent = await commandHandler.handle(
        { state: this._state } as never,
        command
      );

      if (Array.isArray(generatedEvent)) {
        for await (const event of generatedEvent) {
          await this.client.insertEvent(event as never);
        }
      } else {
        await this.client.insertEvent(generatedEvent as never);
      }

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

      for (const event of events) {
        const eventHandler = this.eventHandlers.get(event.type);

        assert(eventHandler, `event handler for ${event.type} not found`);

        const state = await eventHandler.handle(
          { state: this._state } as never,
          event
        );

        this._state = state;
        this._version = event.aggregate.version;
      }

      release();
    } catch (err) {
      release();

      throw err;
    }
  }
}
