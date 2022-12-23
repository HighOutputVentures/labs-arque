import { Client } from './client';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';
import { Backoff, FibonacciStrategy } from 'backoff';
import { InvalidAggregateVersionError } from './error';
import R from 'ramda';
import { Context } from './common';

export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;

  private commandHandlers: Map<number, CommandHandler<TCommand, TEvent, TState, TContext>> =
    new Map();

  private eventHandlers: Map<number, EventHandler<TEvent, TState, TContext>> = new Map();

  constructor(
    private _id: ObjectId,
    private _version: number,
    private _state: TState,
    private client: Client,
    commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[],
    eventHandlers: EventHandler<TEvent, TState, TContext>[],
    private preProcessHook?: (ctx: Context<TState, TContext>) => void | Promise<void>,
    private postProcessHook?: (ctx: Context<TState, TContext>) => void | Promise<void>
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

  private async digest(ctx: Context<TState, TContext>, events: TEvent[]) {
    for (const event of events) {
      const eventHandler = this.eventHandlers.get(event.type);

      assert(eventHandler, `event handler for ${event.type} does not exist`);

      const state = await eventHandler.handle(ctx, event);

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
    const ctx = {
      get state(): TState {
        return this._state;
      },
      get version(): number {
        return this._version;
      }
    } as Context<TState, TContext>;

    const release = await this.mutex.acquire();

    try {
      if (this.preProcessHook) await this.preProcessHook(ctx);

      let events = await this.client.listAggregateEvents<TEvent>({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(ctx, events);

      const commandHandler = this.commandHandlers.get(command.type);

      assert(commandHandler, `command handler for ${command.type} does not exist`);

      const generatedEvent = await commandHandler.handle(
        ctx,
        command
      );

      if (Array.isArray(generatedEvent)) {
        events = await this.client.insertEvents({
          aggregate: {
            id: this._id,
            version: this._version + 1,
          },
          events: R.map(
            (item) => ({
              ...item,
              meta: {},
            }),
            generatedEvent
          ),
        });
      } else {
        const event = await this.client.insertEvent<TEvent>({
          ...generatedEvent,
          aggregate: {
            id: this._id,
            version: this._version + 1,
          },
          meta: {},
        });

        events = [event];
      }

      await this.digest(ctx, events);

      if (this.postProcessHook) await this.postProcessHook(ctx);

      release();
    } catch (err) {
      if (this.postProcessHook) await this.postProcessHook(ctx);

      release();

      throw err;
    }
  }

  public async reload(): Promise<void> {
    const ctx = {
      get state(): TState {
        return this._state;
      },
      get version(): number {
        return this._version;
      }
    } as Context<TState, TContext>;

    const release = await this.mutex.acquire();

    try {
      const events = await this.client.listAggregateEvents({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(ctx, events as TEvent[]);

      release();
    } catch (err) {
      release();

      throw err;
    }
  }
}
