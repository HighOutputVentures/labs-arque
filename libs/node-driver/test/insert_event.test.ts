import { ArqueDriver } from '../src';

describe('fn insertEvent() Test', () => {
  it.concurrent('should print return promise of Event Object', async () => {
    let arqueDriver = new ArqueDriver('tcp://127.0.0.1:4000');

    let response = await arqueDriver.insertEvent({
      id: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
      type_: 1,
      aggregateId: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
      aggregateVersion: 1,
      body: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
      meta: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    });

    expect(response).toEqual(0);
  });
});
