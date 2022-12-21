import { Aggregate } from './aggregate';
import { ObjectId } from './object-id';
import { Command, CommandHandler } from './command';
import { Event, EventHandler } from './event';
import { hash } from 'bcrypt';
import { faker } from '@faker-js/faker';

describe('Aggregate', () => {
  describe('#load', () => {
    type Account = {
      id: ObjectId;
      name: string;
      password: string;
      metadata?: Record<string, unknown>;
      dateTimeCreated: Date;
      dateTimeLastUpdated: Date;
    };

    type AccountAggregateState = Pick<Account, 'dateTimeCreated' | 'dateTimeLastUpdated'>;

    enum EventType {
      AccountCreated = 0,
      AccountUpdated = 1,
    }

    type AccountCreatedEvent = Event<
      EventType.AccountCreated,
      Pick<Account, 'name' | 'password' | 'metadata'>
    >;
    type AccountUpdatedEvent = Event<
      EventType.AccountUpdated,
      Partial<Pick<Account, 'password' | 'metadata'>>
    >;

    test.concurrent('load aggregate instance', async () => {
      const id = new ObjectId();
      const password = await hash(faker.internet.password(), 10);

      const event = {
        id: new ObjectId(),
        aggregate: {
          id,
          version: 2,
        },
        type: EventType.AccountUpdated,
        body: {
          password,
        },
        meta: {},
        timestamp: new Date(),
      };

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([event]),
      };

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            ...ctx.state,
            dateTimeLastUpdated: event.timestamp,
          };
        }),
      };

      const aggregate = new Aggregate(
        {
          commandHandlers: [],
          eventHandlers: [eventHandler],
        },
        ClientMock as never
      );

      await aggregate.load(id);

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);
      expect(ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id).toEqual(id);

      expect(eventHandler.handle).toBeCalledTimes(1);
      expect(eventHandler.handle.mock.calls[0][1]).toEqual(event);
    });
  });
});
