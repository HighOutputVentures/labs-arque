import { Command } from './command';
import { Event } from './event';
import { ObjectId } from './object-id';

export type EventHandler<TState, TContext extends {}> = {
  type: number;
  handle(ctx: TContext & { state: TState }, event: Event): TState | Promise<TState>;
}

type GeneratedEvent<TEvent extends Event> = Pick<TEvent, 'type' | 'body'>;

export type CommandHandler<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  type: number;
  handle(ctx: TContext & { state: TState }, command: TCommand):
    GeneratedEvent<TEvent> |
    GeneratedEvent<TEvent>[] |
    Promise<GeneratedEvent<TEvent>> |
    Promise<GeneratedEvent<TEvent>[]>;
}

export type AggregateOptions<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  eventHandlers: EventHandler<TState, TContext>[];
  commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[];
  preProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
  postProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
}

export class AggregateInstance<
  TCommand extends Command,
  TState,
> {
  constructor(private _id: ObjectId, private _version: number, private _state: TState) {};

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
    throw new Error('not implemented');
  }
}

export class Aggregate<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  constructor(opts: AggregateOptions<TCommand, TEvent, TState, TContext>) {
    throw new Error('not implemented');
  }

  public async load(_id: ObjectId): Promise<AggregateInstance<TCommand, TState>> {
    throw new Error('not implemented');
  }
}
