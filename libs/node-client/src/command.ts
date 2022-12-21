import { Event } from './event';

export type Command<TType extends number = number, TParams extends {} = {}> = {
  type: TType;
  params: TParams;
};

export type GeneratedEvent<TEvent extends Event> = Pick<TEvent, 'type' | 'body'>;

export type CommandHandlerContext<TState, TContext extends {}> = TContext & { state: TState };

export type CommandHandler<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  type: number;
  handle(
    ctx: CommandHandlerContext<TState, TContext>,
    command: TCommand
  ):
    | GeneratedEvent<TEvent>
    | GeneratedEvent<TEvent>[]
    | Promise<GeneratedEvent<TEvent>>
    | Promise<GeneratedEvent<TEvent>[]>;
};
