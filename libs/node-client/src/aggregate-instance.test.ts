import { AggregateInstance } from './aggregate-instance';
import { ObjectId } from './object-id';
import { Command } from './command';
import { Event } from './event';
import { faker } from '@faker-js/faker';
import { hash } from 'bcrypt';

describe('AggregateInstance', () => {
  describe.only('#reload', () => {
    test.concurrent('update to the latest state', async () => {
      type Account = {
        id: ObjectId;
        name: string;
        password: string;
        metadata?: Record<string, unknown>;
        dateTimeCreated: Date;
        dateTimeLastUpdated: Date;
      };
      
      type AccountAggregateState = {
        root: Account,
      };
      
      enum EventType {
        AccountCreated = 0,
        AccountUpdated = 1,
      };
      
      type AccountUpdatedEvent = Event<
        EventType.AccountUpdated,
        Partial<Pick<Account, 'password' | 'metadata'>>
      >;      

      const id = new ObjectId();
      const password = await hash(faker.internet.password(), 10);

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([
          {
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
          },
        ]),
      };

      const eventHandlers = [
        {
          type: EventType.AccountUpdated,
          handle(ctx, event: AccountUpdatedEvent) {
            return {
              root: {
                ...ctx.state.root,
                ...event.body,
                dateTimeLastUpdated: event.timestamp,
              },
            };
          },
        },
      ];

      let aggregate = new AggregateInstance<Command, Event, AccountAggregateState, {}>(
        id,
        1,
        {
          root: {
            id,
            name: faker.name.firstName().toLowerCase(),
            password: await hash(faker.internet.password(), 10),
            metadata: {
              firstName: faker.name.firstName(),
              lastName: faker.name.lastName(),
            },
            dateTimeCreated: new Date(),
            dateTimeLastUpdated: new Date(),
          }
        },
        ClientMock as never,
        [],
        eventHandlers
      );

      await aggregate.reload();

      expect(aggregate.version).toEqual(2);
      expect(aggregate.state.root.password).toEqual(password);
    });

    // should concurrent
    test.todo('no events');
    test.todo('multiple concurrent execution');
  });

  describe('#process', () => {
    // should concurrent
    test.todo('process a command');
    test.todo('invalid command');
    test.todo('invalid aggregate version');
    test.todo('multiple concurrent execution');
  });
});
