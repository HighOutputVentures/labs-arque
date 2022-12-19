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

export type EventHandlerContext<TState, TContext extends {}> = TContext & { state: TState };

export type EventHandler<TEvent extends Event, TState, TContext extends {}> = {
  type: number;
  handle(ctx: EventHandlerContext<TState, TContext>, event: TEvent): TState | Promise<TState>;
};
