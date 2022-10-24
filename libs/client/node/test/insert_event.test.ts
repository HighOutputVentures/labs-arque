const insertEvent = require('..');

describe('fn insertEvent() Test', () => {
  it.concurrent('should print return promise of Event Object', () => {
    const event = insertEvent.insertEvent();

    const event_object = {
      id: [1, 2, 3, 4, 5],
      type: 1,
      timestamp: 12345,
      aggregate_id: [1, 2, 3, 4, 5],
      aggregate_version: 1,
      body: [1, 2, 3, 4, 5],
      metadata: [1, 2, 3, 4, 5],
      version: 1,
    };

    expect(event).resolves.toHaveProperty('id', event_object.id);
    expect(event).resolves.toHaveProperty('type', event_object.type);
    expect(event).resolves.toHaveProperty('timestamp', event_object.timestamp);
    expect(event).resolves.toHaveProperty(
      'aggregate_id',
      event_object.aggregate_id,
    );
    expect(event).resolves.toHaveProperty(
      'aggregate_version',
      event_object.aggregate_version,
    );
    expect(event).resolves.toHaveProperty('body', event_object.body);
    expect(event).resolves.toHaveProperty('metadata', event_object.metadata);
    expect(event).resolves.toHaveProperty('version', event_object.version);
  });
});
