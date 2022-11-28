export enum ResponseStatus {
  Ok = 0,
  InvalidAggregateVersionError = 1,
  BadRequestError = 2,
  UnknownError = 3,
}

export type Event = {
  id: Buffer;
  type_: number;
  aggregateId: Buffer;
  aggregateVersion: number;
  body: Buffer;
  meta: Buffer;
};
