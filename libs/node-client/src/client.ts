import { inject, injectable } from 'inversify';
import { Event } from './event';
import { ObjectId } from './object-id';
import { TYPE } from './type';
import 'reflect-metadata';

export type InsertEventParams<
  TType extends number = number,
  TBody extends Record<string, any> = Record<string, any>,
  TMeta extends Record<string, any> = Record<string, any>
> = Pick<Event<TType, TBody, TMeta>, 'type' | 'aggregate' | 'body' | 'meta'>;

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
  constructor(@inject(TYPE.ClientOptions) opts: ClientOptions) {
    console.log({ opts });
  }

  public async connect() {
    throw new Error('not implemented');
  }

  public async disconnect() {
    throw new Error('not implemented');
  }

  public async insertEvent<
    TType extends number = number,
    TBody extends Record<string, any> = Record<string, any>,
    TMeta extends Record<string, any> = Record<string, any>
  >(
    _params: InsertEventParams<TType, TBody, TMeta>
  ): Promise<Event<TType, TBody, TMeta>> {
    throw new Error('not implemented');
  }

  public async listAggregateEvents<TEvent extends Event = Event>(
    _params: ListAggregateEventsParams
  ): Promise<TEvent[]> {
    return [];
  }
}
