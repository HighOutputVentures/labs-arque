import { AggregateInstance } from './aggregate-instance';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';

export type AggregateOptions<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  eventHandlers: EventHandler<TState, TContext>[];
  commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[];
  // preProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
  // postProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
};

export class Aggregate<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  constructor(opts: AggregateOptions<TCommand, TEvent, TState, TContext>) {
    throw new Error('not implemented');
  }

  public async load(id: ObjectId): Promise<AggregateInstance<TCommand, TEvent, TState, TContext>> {
    throw new Error('not implemented');
  }
}
