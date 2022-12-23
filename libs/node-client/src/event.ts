import { Context } from './common';
import { ObjectId } from './object-id';

export type Event<TType extends number = number, TBody extends {} = {}, TMeta extends {} = {}> = {
  id: ObjectId;
  type: TType;
  aggregate: {
    id: ObjectId;
    version: number;
  };
  body: TBody;
  meta: TMeta;
  timestamp: Date;
};

export type EventHandler<TEvent extends Event, TState, TContext extends {}> = {
  type: number;
  handle(ctx: Context<TState, TContext>, event: TEvent): TState | Promise<TState>;
};
