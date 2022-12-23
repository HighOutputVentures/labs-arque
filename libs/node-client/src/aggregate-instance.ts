import { Client } from './client';
import { Command, CommandHandler, CommandHandlerContext } from './command';
import { Event, EventHandler, EventHandlerContext } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';
import { Backoff, FibonacciStrategy } from 'backoff';
import { InvalidAggregateVersionError } from './error';
import R from 'ramda';

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

  private context: TContext;

  constructor(
    private _id: ObjectId,
    private _version: number,
    private _state: TState,
    private client: Client,
    commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[],
    eventHandlers: EventHandler<TEvent, TState, TContext>[],
    private preProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>,
    private postProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>
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
    this.context = { ...this.context, state: this._state };

    for (const event of events) {
      const eventHandler = this.eventHandlers.get(event.type);

      assert(eventHandler, `event handler for ${event.type} does not exist`);

      const state = await eventHandler.handle(
        this.context as EventHandlerContext<TState, TContext>,
        event
      );

      this._state = state;
      this._version = event.aggregate.version;
      this.context = { ...this.context, state };
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
      this.context = {} as TContext;

      if (this.preProcessHook) await this.preProcessHook(this.context);

      let events = await this.client.listAggregateEvents<TEvent>({
        aggregate: {
          id: this._id,
          version: this._version,
        },
      });

      await this.digest(events);

      const commandHandler = this.commandHandlers.get(command.type);

      assert(commandHandler, `command handler for ${command.type} does not exist`);

      const generatedEvent = await commandHandler.handle(
        this.context as CommandHandlerContext<TState, TContext>,
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

      await this.digest(events);

      if (this.postProcessHook) await this.postProcessHook(this.context);

      release();
    } catch (err) {
      if (this.postProcessHook) await this.postProcessHook(this.context);

      release();

      throw err;
    }
  }

  public async reload(): Promise<void> {
    const release = await this.mutex.acquire();

    try {
      const events = await this.client.listAggregateEvents({
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
