import { ObjectId } from './object-id';

export type Event<
  TType extends number = number,
  TBody extends Record<string, any> = Record<string, any>,
  TMeta extends Record<string, any> = Record<string, any>,
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
