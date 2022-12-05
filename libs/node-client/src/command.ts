export type Command<
  TType extends number = number,
  TParameters extends {} = {},
> = {
  type: TType;
  parameters: TParameters;
}
