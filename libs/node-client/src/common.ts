export type Context<TState, TContext extends {}> = TContext & { readonly state: TState, readonly version: number };