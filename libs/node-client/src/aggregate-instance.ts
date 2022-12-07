import { Client } from './client';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import { Mutex } from 'async-mutex';
export class AggregateInstance<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  private mutex: Mutex;
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
    private eventHandlers: EventHandler<TState, TContext>[]
  ) {
    this.mutex = new Mutex();
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
    await this.mutex.runExclusive(async () => {
      let version = this._version;
      let id = this._id;

      let events = await this.client.listAggregateEvents({
        aggregate: {
          id,
          version,
        },
      });

      events.map((event) => {
        if (version < event.aggregate.version) {
          this._id = event.aggregate.id;
          this._version = event.aggregate.version;
          this._state['root'] = {
            ...event.body,
            metadata: event.meta,
          };
        }
      });
    });
  }
}
