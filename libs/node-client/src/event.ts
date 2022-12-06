import { ObjectId } from './object-id';

export type Event<
  TType extends number = number,
  TBody extends {} = {},
  TMeta extends {} = {},
> = {
  id: ObjectId;
  type: TType;
  aggregate: {
    id: ObjectId;
    version: number;
  };
  body: TBody;
  meta: TMeta;
  timestamp: Date;
}

export type EventHandler<TState, TContext extends {}> = {
  type: number;
  handle(ctx: TContext & { state: TState }, event: Event): TState | Promise<TState>;
}