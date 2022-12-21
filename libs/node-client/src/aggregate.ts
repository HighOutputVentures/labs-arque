import { AggregateInstance } from './aggregate-instance';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { ObjectId } from './object-id';
import LRUCache from 'lru-cache';
import { Client } from './client';
export type AggregateOptions<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  eventHandlers: EventHandler<TEvent, TState, TContext>[];
  commandHandlers: CommandHandler<TCommand, TEvent, TState, TContext>[];
  preProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
  postProcessHook?: <TContext extends {} = {}>(ctx: TContext) => void | Promise<void>;
};

export class Aggregate<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> {
  cache: LRUCache<string, AggregateInstance<TCommand, TEvent, TState, TContext>>;
  constructor(
    private opts: AggregateOptions<TCommand, TEvent, TState, TContext>,
    private client: Client
  ) {
    this.cache = new LRUCache<string, AggregateInstance<TCommand, TEvent, TState, TContext>>(
      Object.freeze({
        max: 1024,
        ttl: 1800000,
      })
    );
  }

  public async load(
    id: ObjectId,
    opts?: { noReload?: boolean }
  ): Promise<AggregateInstance<TCommand, TEvent, TState, TContext>> {
    let aggregateInstance = this.cache.get(id.toString());

    if (!aggregateInstance) {
      aggregateInstance = new AggregateInstance(
        id,
        0,
        null,
        this.client,
        this.opts.commandHandlers,
        this.opts.eventHandlers,
        this.opts.preProcessHook,
        this.opts.postProcessHook
      );
      this.cache.set(id.toString(), aggregateInstance);
    }

    if (opts && opts.noReload) return aggregateInstance;

    await aggregateInstance.reload();

    return aggregateInstance;
  }
}
