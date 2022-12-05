import { Arque } from '../src/arque';
import { Command } from '../src/command';
import { Event } from '../src/event';
import { ObjectId } from '../src/object-id';

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
}

enum CommandType {
  CreateAccount = 0,
  UpdateAccount = 1,
}

type CreateAccountCommand = Command<
  CommandType.CreateAccount,
  Pick<Account, 'name' | 'password' | 'metadata'>
>;

type UpdateAccountCommand = Command<
  CommandType.UpdateAccount,
  Partial<Pick<Account, 'password' | 'metadata'>>
>;

type AccountAggregateCommand = CreateAccountCommand | UpdateAccountCommand;

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

type AccountAggregateEvent = AccountCreatedEvent | AccountUpdatedEvent;

async function main() {
  const arque = new Arque({
    url: 'arque://localhost:3421',
  });

  await arque.connect();

  const AccountAggregate = arque.aggregate<
    AccountAggregateCommand,
    AccountAggregateEvent,
    AccountAggregateState
  >({
    commandHandlers: [
      {
        type: CommandType.CreateAccount,
        handle(state, command: CreateAccountCommand) {
          if (state) {
            throw new Error('account already exists');
          }

          return {
            type: EventType.AccountCreated,
            body: command.parameters
          };
        },
      },
      {
        type: CommandType.UpdateAccount,
        handle(_, command: UpdateAccountCommand) {
          return {
            type: EventType.AccountUpdated,
            body: command.parameters
          }
        },
      }
    ],
    eventHandlers: [
      {
        type: EventType.AccountCreated,
        handle(_, event: AccountCreatedEvent) {
          return {
            root: {
              id: event.aggregate.id,
              name: event.body.name,
              password: event.body.password,
              dateTimeCreated: event.timestamp,
              dateTimeLastUpdated: event.timestamp,
            }
          };
        }
      },
      {
        type: EventType.AccountUpdated,
        handle(state, event: AccountUpdatedEvent) {
          return {
            root: {
              ...state.root,
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            }
          };
        }
      }
    ]
  });

  const id = ObjectId.from('2a65d66ced4b8adf4f1ecd79');

  const aggregate = await AccountAggregate.load(id);

  await aggregate.process({
    type: CommandType.CreateAccount,
    parameters: {
      name: 'user',
      password: 'password',
      metadata: {
        emailAddress: 'user@arque.io'
      }
    }
  });

  await aggregate.process({
    type: CommandType.UpdateAccount,
    parameters: {
      password: 'password1',
    }
  });

  await aggregate.reload();

  console.log(aggregate.state);

  await arque.disconnect();
}

main();
