import { Client } from './client';
import { Command, CommandHandler, GeneratedEvent } from './command';
import { Event, EventHandler, EventHandlerContext } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';
import { Backoff, FibonacciStrategy } from 'backoff';
import { InvalidAggregateVersionError } from './error';
import R, { last } from 'ramda';
import Queue from 'p-queue';

export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;
  private eventHandlers: Map<number, EventHandler<TState, TContext>> =
    new Map();

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

      assert(eventHandler, `event handler for ${event.type} does not exist`);

      const state = await eventHandler.handle(
        {
          state: this._state,
        } as EventHandlerContext<TState, TContext>,
        event
      );

      this._state = state;
      this._version = event.aggregate.version;
    }
  }

  public async process(command: TCommand): Promise<void> {
    const backoff = new Backoff(
      new FibonacciStrategy({
        randomisationFactor: 0.2,
        initialDelay: 100,
        maxDelay: 5000,
      })
    );

    backoff.failAfter(10);

    return new Promise((resolve, reject) => {
      backoff.on('ready', async () => {
        try {
          await this.processCommand(command);

          resolve();
        } catch (err) {
          if (err instanceof InvalidAggregateVersionError) {
            backoff.backoff(err);
          }

          reject(err);
        }
      });

      backoff.once('fail', function (err) {
        reject(err);
      });

      backoff.backoff();
    });
  }

  private async processCommand(command: TCommand): Promise<void> {
    const release = await this.mutex.acquire();

    try {
      let events = await this.client.listAggregateEvents<TEvent>({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(events);

      const commandHandler = this.commandHandlers.get(command.type);

      assert(
        commandHandler,
        `command handler for ${command.type} does not exist`
      );

      const generatedEvent = await commandHandler.handle(
        {
          state: this._state,
        } as TContext & { state: TState },
        command
      );

      if (Array.isArray(generatedEvent)) {
        const queue = new Queue();

        for await (const _generatedEvent of generatedEvent) {
          await queue.add(async () => {
            const params = {
              ..._generatedEvent,
              aggregate: {
                id: this._id,
                version: this._version + 1,
              },
              meta: {},
            };

            const event = await this.client.insertEvent<TEvent>(params);

            await this.digest([event]);
          });
        }

        await queue.onIdle();
      } else {
        const params = {
          ...generatedEvent,
          aggregate: {
            id: this._id,
            version: this._version + 1,
          },
          meta: {},
        };

        const event = await this.client.insertEvent<TEvent>(params);

        await this.digest([event]);
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

      await this.digest(events as TEvent[]);

      release();
    } catch (err) {
      release();

      throw err;
    }
  }
}
