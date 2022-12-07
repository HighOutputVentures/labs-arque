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
      const id = ObjectId.from('2a65d66ced4b8adf4f1ecd79');

      const clientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([
          {
            id: new ObjectId(),
            aggregate: {
              id: new ObjectId(),
              version: 2,
            },
            type: 1,
            body: {
              id: new ObjectId(),
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
        id,
        1,
        { root: {} },
        clientMock as never,
        commandHandlers,
        eventHandlers
      );

      await aggregateInstance.reload();

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
