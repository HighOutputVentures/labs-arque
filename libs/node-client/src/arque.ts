import { Aggregate, AggregateOptions } from './aggregate';
import { Command } from './command';
import { Event } from './event';

export type ArqueOptions = {
  url?: string;
};

export type AggregateParams<
  TCommand extends Command,
  TEvent extends Event,
  TState = unknown
> = Pick<AggregateOptions<TCommand, TEvent, TState>, 'eventHandlers' | 'commandHandlers'>;

export class Arque {
  constructor(opts: ArqueOptions) {
    throw new Error('not implemented');
  }

  public async connect() {
    throw new Error('not implemented');
  }

  public async disconnect() {
    throw new Error('not implemented');
  }

  aggregate<
    TCommand extends Command = Command,
    TEvent extends Event = Event,
    TState = unknown
  >(params: AggregateParams<TCommand, TEvent, TState>): Aggregate<TCommand, TEvent, TState> {
    throw new Error('not implemented');
  }
}
