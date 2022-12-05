import { Event } from './event';
import { ObjectId } from './object-id';

export type InsertEventParams<
  TType extends number = number,
  TBody extends Record<string, any> = Record<string, any>,
  TMeta extends Record<string, any> = Record<string, any>,
> = Pick<Event<TType, TBody, TMeta>, 'type' | 'aggregate' | 'body' | 'meta'>;

export type ListAggregateEventsParams = {
  aggregate: {
    id: ObjectId;
    version?: number;
  },
  limit?: number;
};

export class Client {
  public async connect() {
    throw new Error('not implemented');
  }

  public async disconnect() {
    throw new Error('not implemented');
  }

  public async insertEvent<
    TType extends number = number,
    TBody extends Record<string, any> = Record<string, any>,
    TMeta extends Record<string, any> = Record<string, any>,
  > (_params: InsertEventParams<TType, TBody, TMeta>): Promise<Event<TType, TBody, TMeta>> {
    throw new Error('not implemented');
  }

  public async listAggregateEvents<T extends Event>(_params: ListAggregateEventsParams): Promise<T> {
    throw new Error('not implemented');
  }
}
