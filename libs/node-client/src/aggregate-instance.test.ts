import { AggregateInstance } from './aggregate-instance';
import { ObjectId } from './object-id';
import { Command } from './command';
import { Event } from './event';
import { faker } from '@faker-js/faker';
import { hash } from 'bcrypt';
import { InvalidAggregateVersionError } from './error';

describe('AggregateInstance', () => {
  type Account = {
    id: ObjectId;
    name: string;
    password: string;
    metadata?: Record<string, unknown>;
    dateTimeCreated: Date;
    dateTimeLastUpdated: Date;
  };

  type AccountAggregateState = {
    root: Account;
  };

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

  describe('#reload', () => {
    test.concurrent('update to the latest state', async () => {
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
            root: {
              ...ctx.state.root,
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const version = 1;
      const state = {
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
        },
      };

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(id, version, state, ClientMock as never, [], [eventHandler]);

      await aggregate.reload();

      expect(aggregate.version).toEqual(2);
      expect(aggregate.state.root.password).toEqual(password);

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);

      expect(eventHandler.handle).toBeCalledTimes(1);
      expect(eventHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
      expect(eventHandler.handle.mock.calls[0][1]).toEqual(event);
    });

    // should concurrent
    test.concurrent('no events', async () => {
      const id = new ObjectId();

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([]),
      };

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            root: {
              ...ctx.state.root,
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const version = 1;
      const state = {
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
        },
      };

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(id, version, state, ClientMock as never, [], [eventHandler]);

      await aggregate.reload();

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);

      expect(eventHandler.handle).not.toBeCalled();
    });

    test.concurrent('multiple concurrent execution', async () => {
      const id = new ObjectId();
      const password = await hash(faker.internet.password(), 10);

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            root: {
              ...ctx.state.root,
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const state = {
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
        },
      };

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

      const ClientMockV2 = {
        listAggregateEvents: jest.fn().mockResolvedValue([
          {
            id: new ObjectId(),
            aggregate: {
              id,
              version: 5,
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

      const [firstVersion, secondVersion] = await Promise.all([
        (async () => {
          let aggregate = new AggregateInstance<
            Command,
            Event,
            AccountAggregateState,
            {}
          >(id, 1, state, ClientMock as never, [], [eventHandler]);

          await aggregate.reload();

          return aggregate.version;
        })(),

        (async () => {
          let aggregateV2 = new AggregateInstance<
            Command,
            Event,
            AccountAggregateState,
            {}
          >(id, 2, state, ClientMockV2 as never, [], [eventHandler]);

          await aggregateV2.reload();

          return aggregateV2.version;
        })(),
      ]);

      expect(firstVersion).toBeLessThan(secondVersion);

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);

      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(1);

      expect(ClientMockV2.listAggregateEvents).toBeCalledTimes(1);

      expect(
        ClientMockV2.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMockV2.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(2);

      expect(eventHandler.handle).toBeCalledTimes(2);
      expect(eventHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
    });
  });

  describe('#process', () => {
    // should concurrent
    test.concurrent('process a command', async () => {
      const id = new ObjectId();

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([]),
        insertEvent: jest.fn().mockImplementation(async (params) => params),
      };

      const eventHandler = {
        type: EventType.AccountCreated,
        handle: jest.fn(async (ctx, event: AccountCreatedEvent) => {
          return {
            root: {
              ...(ctx.state ? ctx.state.root : {}),
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const commandHandler = {
        type: CommandType.CreateAccount,
        handle: jest.fn(async (ctx, command: CreateAccountCommand) => {
          if (ctx.state) {
            throw new Error('account already exists');
          }
          return {
            type: EventType.AccountCreated,
            body: command.params,
          };
        }),
      };

      const version = 0;

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(
        id,
        version,
        null,
        ClientMock as never,
        [commandHandler],
        [eventHandler]
      );

      const command = {
        type: CommandType.CreateAccount,
        params: {
          name: faker.name.firstName().toLowerCase(),
          password: await hash(faker.internet.password(), 10),
          metadata: {
            firstName: faker.name.firstName(),
            lastName: faker.name.lastName(),
          },
        },
      };

      await aggregate.process(command);

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);
      expect(eventHandler.handle).toBeCalledTimes(1);
      expect(ClientMock.insertEvent).toBeCalledTimes(1);
      expect(commandHandler.handle).toBeCalledTimes(1);
      expect(commandHandler.handle.mock.calls[0][0].state).toBeNull();
      expect(commandHandler.handle.mock.calls[0][1]).toEqual(command);
    });
    test.concurrent('invalid command', async () => {
      const id = new ObjectId();
      const event = {
        id: new ObjectId(),
        aggregate: {
          id,
          version: 2,
        },
        type: EventType.AccountUpdated,
        body: {
          password: await hash(faker.internet.password(), 10),
        },
        meta: {},
        timestamp: new Date(),
      };

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([event]),
        insertEvent: jest.fn().mockResolvedValue(null),
      };

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            root: {
              ...(ctx.state ? ctx.state.root : {}),
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const commandHandler = {
        type: CommandType.CreateAccount,
        handle: jest.fn(async (ctx, command: CreateAccountCommand) => {
          if (ctx.state) {
            throw new Error('account already exists');
          }

          return {
            type: EventType.AccountCreated,
            body: command.params,
          };
        }),
      };

      const version = 1;
      const state = {
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
        },
      };

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(
        id,
        version,
        state,
        ClientMock as never,
        [commandHandler],
        [eventHandler]
      );

      const command = {
        type: CommandType.CreateAccount,
        params: {
          name: faker.name.firstName().toLowerCase(),
          password: await hash(faker.internet.password(), 10),
          metadata: {
            firstName: faker.name.firstName(),
            lastName: faker.name.lastName(),
          },
        },
      };

      await expect(aggregate.process(command)).rejects.toThrow(
        'account already exists'
      );

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);

      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);

      expect(ClientMock.insertEvent).not.toBeCalled();

      expect(eventHandler.handle).toBeCalledTimes(1);
      expect(eventHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
      expect(eventHandler.handle.mock.calls[0][1]).toEqual(event);

      expect(commandHandler.handle).toBeCalledTimes(1);
      expect(commandHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
      expect(commandHandler.handle.mock.calls[0][1]).toEqual(command);
    });
    test.concurrent('invalid aggregate version', async () => {
      const id = new ObjectId();
      const version = 2;

      const ClientMock = {
        listAggregateEvents: jest.fn().mockResolvedValue([]),
        insertEvent: jest.fn().mockRejectedValue(
          new InvalidAggregateVersionError({
            aggregate: id,
            currentVersion: 1,
            nextVersion: version,
          })
        ),
      };

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            root: {
              ...(ctx.state ? ctx.state.root : {}),
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const commandHandler = {
        type: CommandType.UpdateAccount,
        handle: jest.fn(async (ctx, command: UpdateAccountCommand) => {
          return {
            type: EventType.AccountUpdated,
            body: command.params,
          };
        }),
      };

      const state = {
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
        },
      };

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(
        id,
        version,
        state,
        ClientMock as never,
        [commandHandler],
        [eventHandler]
      );

      const command = {
        type: CommandType.UpdateAccount,
        params: {
          name: faker.name.firstName().toLowerCase(),
          password: await hash(faker.internet.password(), 10),
        },
      };

      await expect(aggregate.process(command)).rejects.toThrow(
        'invalid aggregate version'
      );

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(1);

      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);

      expect(eventHandler.handle).not.toBeCalled();

      expect(commandHandler.handle).toBeCalledTimes(1);
      expect(commandHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
      expect(commandHandler.handle.mock.calls[0][1]).toEqual(command);
    });
    test.concurrent('multiple concurrent execution', async () => {
      const id = new ObjectId();

      let currentVersion = 1;
      let currentPassword = await hash(faker.internet.password(), 10);

      const ClientMock = {
        listAggregateEvents: jest.fn().mockImplementation(async (_params) => {
          return [
            {
              id: new ObjectId(),
              aggregate: {
                id,
                version: currentVersion,
              },
              type: EventType.AccountUpdated,
              body: {
                password: currentPassword,
              },
              meta: {},
              timestamp: new Date(),
            },
          ];
        }),
        insertEvent: jest.fn().mockImplementation(async (event: Event) => {
          currentVersion = event.aggregate.version;
          currentPassword = event.body['password'];

          return event;
        }),
      };

      const eventHandler = {
        type: EventType.AccountUpdated,
        handle: jest.fn(async (ctx, event: AccountUpdatedEvent) => {
          return {
            root: {
              ...(ctx.state ? ctx.state.root : {}),
              ...event.body,
              dateTimeLastUpdated: event.timestamp,
            },
          };
        }),
      };

      const commandHandler = {
        type: CommandType.UpdateAccount,
        handle: jest.fn(async (ctx, command: UpdateAccountCommand) => {
          return {
            type: EventType.AccountUpdated,
            body: command.params,
          };
        }),
      };

      const version = 1;
      const state = {
        root: {
          id,
          name: faker.name.firstName().toLowerCase(),
          password: currentPassword,
          metadata: {
            firstName: faker.name.firstName(),
            lastName: faker.name.lastName(),
          },
          dateTimeCreated: new Date(),
          dateTimeLastUpdated: new Date(),
        },
      };

      let aggregate = new AggregateInstance<
        Command,
        Event,
        AccountAggregateState,
        {}
      >(
        id,
        version,
        state,
        ClientMock as never,
        [commandHandler],
        [eventHandler]
      );

      const firstCommand = {
        type: CommandType.UpdateAccount,
        params: {
          password: await hash(faker.internet.password(), 10),
        },
      };

      const secondCommand = {
        type: CommandType.UpdateAccount,
        params: {
          password: await hash(faker.internet.password(), 10),
        },
      };

      await Promise.all([
        (async () => {
          await aggregate.process(firstCommand);
        })(),
        (async () => {
          await aggregate.process(secondCommand);
        })(),
      ]);

      expect(ClientMock.listAggregateEvents).toBeCalledTimes(2);

      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[0][0].aggregate.version
      ).toEqual(version);

      expect(
        ClientMock.listAggregateEvents.mock.calls[1][0].aggregate.id
      ).toEqual(id);
      expect(
        ClientMock.listAggregateEvents.mock.calls[1][0].aggregate.version
      ).toEqual(version + 1);

      expect(eventHandler.handle).toBeCalledTimes(4);

      expect(ClientMock.insertEvent).toBeCalledTimes(2);

      expect(commandHandler.handle).toBeCalledTimes(2);

      expect(commandHandler.handle.mock.calls[0][0].state.root.id).toEqual(id);
      expect(commandHandler.handle.mock.calls[0][1]).toEqual(firstCommand);

      expect(commandHandler.handle.mock.calls[1][0].state.root.id).toEqual(id);
      expect(commandHandler.handle.mock.calls[1][1]).toEqual(secondCommand);
    });
  });
});
