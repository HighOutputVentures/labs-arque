import { inject, injectable } from 'inversify';
import { Event } from './event';
import { ObjectId } from './object-id';
import { TYPE } from './type';
import 'reflect-metadata';

export type InsertEventParams<TEvent extends Event> = Pick<
  TEvent,
  'type' | 'aggregate' | 'body' | 'meta'
>;

export type InsertEventsParams<TEvent extends Event> = Pick<TEvent, 'aggregate'> & {
  events: Pick<TEvent, 'type' | 'body' | 'meta'>[];
};

export type ListAggregateEventsParams = {
  aggregate: {
    id: ObjectId;
    version?: number;
  };
  limit?: number;
};

export type ClientOptions = {
  url?: string;
};

@injectable()
export class Client {
  constructor(@inject(TYPE.ClientOptions) _opts: ClientOptions) {
    throw new Error('not implemented');
  }

  public async connect() {
    throw new Error('not implemented');
  }

  public async disconnect() {
    throw new Error('not implemented');
  }

  public async insertEvent<TEvent extends Event = Event>(
    _params: InsertEventParams<TEvent>
  ): Promise<TEvent> {
    throw new Error('not implemented');
  }

  public async insertEvents<TEvent extends Event = Event>(
    _params: InsertEventsParams<TEvent>
  ): Promise<TEvent[]> {
    throw new Error('not implemented');
  }

  public async listAggregateEvents<TEvent extends Event = Event>(
    _params: ListAggregateEventsParams
  ): Promise<TEvent[]> {
    return [];
  }
}
