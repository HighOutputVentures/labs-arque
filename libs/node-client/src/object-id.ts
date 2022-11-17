export class ObjectId {
  static from(value: Buffer | string): ObjectId {
    throw new Error('not implemented');
  }

  public toBuffer(): Buffer {
    throw new Error('not implemented');
  }

  public toString(): string {
    throw new Error('not implemented');
  }
}