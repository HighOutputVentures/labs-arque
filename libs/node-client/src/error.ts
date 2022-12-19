import { ObjectId } from './object-id';

export class InvalidAggregateVersionError extends Error {
  currentVersion: number;
  nextVersion: number;
  aggregate: ObjectId;

  constructor(params: { aggregate: ObjectId; currentVersion: number; nextVersion: number }) {
    super('invalid aggregate version');

    this.aggregate = params.aggregate;
    this.currentVersion = params.currentVersion;
    this.nextVersion = params.nextVersion;
  }
}
