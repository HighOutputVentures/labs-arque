import { Client } from './client';
import { Command } from './command';
import { ObjectId } from './object-id';

export class AggregateInstance<
  TCommand extends Command,
  TState,
> {
  constructor(
    private _id: ObjectId,
    private _version: number,
    private _state: TState,
    private client: Client,
  ) {};

  get id() {
    return this._id;
  }

  get version() {
    return this._version;
  }

  get state() {
    return this._state;
  }

  public async process(command: TCommand): Promise<void> {
    throw new Error('not implemented');
  }

  public async reload(): Promise<void> {
    throw new Error('not implemented');
  }
}