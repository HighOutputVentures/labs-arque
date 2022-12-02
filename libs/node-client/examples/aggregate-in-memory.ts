import { Arque } from '../src/arque';
import { Command } from '../src/command';
import { Event } from '../src/event';
import { ObjectId } from '../src/object-id';

type AccountAggregateState = {
  account: {
    id: ObjectId;
    name: string;
    password: string;
    metadata?: Record<string, unknown>;
    dateTimeCreated: Date;
    dateTimeLastUpdated: Date;
  }
}

enum CommandType {
  CreateAccount = 0,
  UpdateAccount = 1,
}

type CreateAccountCommand = Command<
  CommandType.CreateAccount,
  Pick<AccountAggregateState['account'], 'name' | 'password' | 'metadata'>
>;

type UpdateAccountCommand = Command<
  CommandType.UpdateAccount,
  Partial<Pick<AccountAggregateState['account'], 'password' | 'metadata'>>
>;

type AccountAggregateCommand = CreateAccountCommand | UpdateAccountCommand;

enum EventType {
  AccountCreated = 0,
  AccountUpdated = 1,
}

type AccountCreatedEvent = Event<
  EventType.AccountCreated,
  Pick<AccountAggregateState['account'], 'name' | 'password' | 'metadata'>
>;

type AccountUpdatedEvent = Event<
  EventType.AccountUpdated,
  Partial<Pick<AccountAggregateState['account'], 'password' | 'metadata'>>
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
        apply(_, event: AccountCreatedEvent) {
          return {
            account: {
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
        apply(state, event: AccountUpdatedEvent) {
          return {
            account: {
              ...state.account,
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

  await arque.disconnect();
}

main();
