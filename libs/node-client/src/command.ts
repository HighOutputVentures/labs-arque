import { Event } from './event';

export type Command<
  TType extends number = number,
  TParams extends {} = {},
> = {
  type: TType;
  params: TParams;
}

export type GeneratedEvent<TEvent extends Event> = Pick<TEvent, 'type' | 'body'>;

export type CommandHandler<
  TCommand extends Command,
  TEvent extends Event,
  TState,
  TContext extends {}
> = {
  type: number;
  handle(ctx: TContext & { state: TState }, command: TCommand):
    GeneratedEvent<TEvent> |
    GeneratedEvent<TEvent>[] |
    Promise<GeneratedEvent<TEvent>> |
    Promise<GeneratedEvent<TEvent>[]>;
}