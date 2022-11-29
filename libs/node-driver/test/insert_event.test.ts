import { ArqueDriver, ResponseStatus } from '../src';

describe('fn insertEvent() Test', () => {
  it.concurrent('should return response status Ok', async () => {
    let arqueDriver = new ArqueDriver('tcp://127.0.0.1:4000');

    let responseStatus = await arqueDriver.insertEvent({
      id: Buffer.from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
      type_: 1,
      aggregateId: Buffer.from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
      aggregateVersion: 1,
      body: Buffer.from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
      meta: Buffer.from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
    });

    expect(responseStatus).toEqual(ResponseStatus.Ok);
  });
});
