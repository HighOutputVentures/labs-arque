import { Aggregate, AggregateOptions } from './aggregate';
import { Command } from './command';
import { Event } from './event';
import { TYPE } from './type';
import { Container } from 'inversify';
import * as R from 'ramda';
import { Client } from './client';

export type ArqueOptions = {
  url?: string;
};

export type AggregateParams<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = Pick<AggregateOptions<TCommand, TEvent, TState, TContext>, 'eventHandlers' | 'commandHandlers'>;

export class Arque {
  private container: Container;

  constructor(opts: ArqueOptions) {
    const container = new Container();

    container.bind(TYPE.ClientOptions).toConstantValue(R.pick(['url'], opts));
    container.bind(TYPE.Client).to(Client);
  }

  public async connect() {
    const client = this.container.get<Client>(TYPE.Client);
    await client.connect();
  }

  public async disconnect() {
    const client = this.container.get<Client>(TYPE.Client);
    await client.disconnect();
  }

  aggregate<
    TCommand extends Command = Command,
    TEvent extends Event = Event,
    TState = unknown,
    TContext extends {} = {}
  >(params: AggregateParams<TCommand, TEvent, TState, TContext>): Aggregate<TCommand, TEvent, TState, TContext> {
    throw new Error('not implemented');
  }
}
