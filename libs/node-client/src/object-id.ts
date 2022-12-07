import { randomBytes } from 'crypto';

let PROCESS_ID: Uint8Array | null = null;
export class ObjectId {
  private static INDEX = Math.floor(Math.random() * 0xffffff);
  private data: Buffer;
  constructor(arg?: Buffer) {
    if (arg) {
      this.data = arg;
    } else {
      const buffer = Buffer.alloc(12);

      const time = Math.floor(Date.now() / 1000);

      if (PROCESS_ID === null) {
        PROCESS_ID = randomBytes(5);
      }

      const increment = ObjectId.getIncrement();

      const timestamp = new DataView(
        buffer.buffer,
        buffer.byteOffset,
        buffer.length
      ).setUint32(0, time, false);

      buffer[4] = PROCESS_ID[0];
      buffer[5] = PROCESS_ID[1];
      buffer[6] = PROCESS_ID[2];
      buffer[7] = PROCESS_ID[3];
      buffer[8] = PROCESS_ID[4];

      buffer[9] = (increment >> 16) & 0xff;
      buffer[10] = (increment >> 8) & 0xff;
      buffer[11] = increment & 0xff;

      this.data = buffer;
    }
  }

  private static getIncrement(): number {
    return (ObjectId.INDEX = (ObjectId.INDEX + 1) % 0xffffff);
  }

  static from(value: Buffer | string): ObjectId {
    if (value instanceof Buffer) return new ObjectId(value);
  }

  public toBuffer(): Buffer {
    return this.data;
  }

  public toString(): string {
    return this.data.toString('hex');
  }

  public equals(other: ObjectId): boolean {
    return this.toBuffer().equals(other.toBuffer());
  }
}
