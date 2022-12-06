export type Command<
  TType extends number = number,
  TParams extends {} = {},
> = {
  type: TType;
  params: TParams;
}
