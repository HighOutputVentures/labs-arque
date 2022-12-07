import { AggregateInstance } from './aggregate-instance';
import { ObjectId } from './object-id';
import { Command } from './command';
import { Event } from './event';

type Account = {
  id: ObjectId;
  name: string;
  password: string;
  metadata?: Record<string, unknown>;
  dateTimeCreated: Date;
  dateTimeLastUpdated: Date;
};

enum CommandType {
  CreateAccount = 0,
  UpdateAccount = 1,
}

type UpdateAccountCommand = Command<
  CommandType.UpdateAccount,
  Partial<Pick<Account, 'password' | 'metadata'>>
>;

enum EventType {
  AccountCreated = 0,
  AccountUpdated = 1,
}
type AccountUpdatedEvent = Event<
  EventType.AccountUpdated,
  Partial<Pick<Account, 'password' | 'metadata'>>
>;

describe('AggregateInstance', () => {
  describe.only('#reload', () => {
    test.concurrent('update to the latest state', async () => {
      const aggregateId = new ObjectId();
      const accountId = new ObjectId();

      const clientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([
          {
            id: new ObjectId(),
            aggregate: {
              id: aggregateId,
              version: 2,
            },
            type: 1,
            body: {
              id: accountId,
              name: 'user',
              password: 'password',
            },
            meta: {
              emailAddress: 'user@arque.io',
            },
            timestamp: new Date(),
          },
        ]),
      };

      const commandHandlers = [
        {
          type: CommandType.UpdateAccount,
          handle(_, command: UpdateAccountCommand) {
            return {
              type: EventType.AccountUpdated,
              body: command.params,
            };
          },
        },
      ];

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

      let aggregateInstance = new AggregateInstance(
        aggregateId,
        1,
        {
          root: {
            name: 'user',
            password: 'zero',
            metadata: {
              emailAddress: 'user@arque.io',
            },
            dateTimeCreated: new Date(),
            dateTimeLastUpdated: new Date(),
          },
        },
        clientMock as never,
        commandHandlers,
        eventHandlers
      );

      await aggregateInstance.reload();

      expect(aggregateInstance.id).toEqual(aggregateId);
      expect(aggregateInstance.version).toEqual(2);
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
