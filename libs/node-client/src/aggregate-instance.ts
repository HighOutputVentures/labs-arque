import { Client } from './client';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
import assert from 'assert';

export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;
  private eventHandlers: Map<number, EventHandler<TState, TContext>> = new Map();

  constructor(
    private _id: ObjectId,
    private _version: number,
    private _state: TState,
    private client: Client,
    private commandHandlers: CommandHandler<
      TCommand,
      TEvent,
      TState,
      TContext
    >[],
    eventHandlers: EventHandler<TState, TContext>[]
  ) {
    this.mutex = new Mutex();

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

  public async process(command: TCommand): Promise<void> {
    throw new Error('not implemented');
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

        const state = await eventHandler.handle({ state: this._state} as never, event);

        this._state = state;
        this._version = event.aggregate.version;
      }

      release();
    } catch(err) {
      release();

      throw err;
    }
  }
}
