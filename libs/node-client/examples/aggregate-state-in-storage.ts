import { ObjectId } from '../src/object-id';
import { Command } from '../src/command';
import { Arque } from '../src/arque';
import { Event } from '../src/event';
import R from 'ramda';

type KanbanBoard = {
  id: ObjectId;
  name: string;
  dateTimeCreated: Date;
  dateTimeLastUpdated: Date;
};

type KanbanBoardColumn = {
  id: ObjectId;
  name: string;
  cardCount: number;
  minimumCardCount?: number;
  maximumCardCount?: number;
};

type KanbanBoardCard = {
  id: ObjectId;
  column: ObjectId;
  name: string;
  description: string;
  dateTimecreated: Date;
  dateTimeLastUpdated: Date;
}

type KanbanBoardAggregateState = {
  root: KanbanBoard,
  columns: KanbanBoardColumn[],
}

enum CommandType {
  CreateKanbanBoard = 0,
  CreateKanbanBoardColumn = 1,
  CreateKanbanBoardCard = 2,
}

type CreateKanbanBoardCommand = Command<
  CommandType.CreateKanbanBoard,
  Pick<KanbanBoard, 'name'>
>;

type CreateKanbanBoardColumnCommand = Command<
  CommandType.CreateKanbanBoardColumn,
  Pick<KanbanBoardColumn, 'id' | 'name' | 'minimumCardCount' | 'maximumCardCount'>
>;

type CreateKanbanBoardCardCommand = Command<
  CommandType.CreateKanbanBoardCard,
  Pick<KanbanBoardCard, 'id' | 'column' | 'name' | 'description'>
>;

type KanbanBoardAggregateCommand = CreateKanbanBoardCommand | CreateKanbanBoardColumnCommand | CreateKanbanBoardCardCommand;

enum EventType {
  KanbanBoardCreated = 0,
  KanbanBoardColumnCreated = 1,
}

type KanbanBoardCreatedEvent = Event<
  EventType.KanbanBoardCreated,
  Pick<KanbanBoard, 'name'>
>;

type KanbanBoardColumnCreatedEvent = Event<
  EventType.KanbanBoardColumnCreated,
  Pick<KanbanBoardColumn, 'id' | 'name' | 'minimumCardCount' | 'maximumCardCount'>
>;

type KanbanBoardAggregateEvent = KanbanBoardCreatedEvent | KanbanBoardColumnCreatedEvent;

async function main() {
  const arque = new Arque({
    url: 'arque://localhost:3421',
  });

  await arque.connect();

  const KanbanBoardAggregate = arque.aggregate<
    KanbanBoardAggregateCommand,
    KanbanBoardAggregateEvent,
    KanbanBoardAggregateState
  >({
    commandHandlers: [
      {
        type: CommandType.CreateKanbanBoard,
        handle(ctx, command: CreateKanbanBoardCommand) {
          if (ctx.state) {
            throw new Error('Kanban board already exists');
          }

          return {
            type: EventType.KanbanBoardCreated,
            body: command.parameters
          };
        },
      },
      {
        type: CommandType.CreateKanbanBoardColumn,
        handle(ctx, command: CreateKanbanBoardColumnCommand) {
          if (ctx.state.columns.length >= 11) {
            throw new Error('can only create up to 12 columns');
          }

          return {
            type: EventType.KanbanBoardColumnCreated,
            body: command.parameters
          };
        },
      }
    ],
    eventHandlers: [
      {
        type: EventType.KanbanBoardCreated,
        handle(state, event: KanbanBoardCreatedEvent) {
          if (state) {
            throw new Error('Kanban board already exists');
          }

          return {
            root: {
              id: event.id,
              name: event.body.name,
              dateTimeCreated: event.timestamp,
              dateTimeLastUpdated: event.timestamp,
            },
            columns: [],
          };
        }
      },
      {
        type: EventType.KanbanBoardColumnCreated,
        handle(ctx, event: KanbanBoardColumnCreatedEvent) {
          const column = R.find((item) => item.id.equals(event.body.id), ctx.state.columns);

          if (column) {
            throw new Error('column already exists');
          }

          return {
            ...ctx.state,
            columns: [
              ...ctx.state.columns,
              {
                ...event.body,
                cardCount: 0,
              }
            ]
          };
        }
      }
    ]
  });

  const id = ObjectId.from('2a65d66ced4b8adf4f1ecd79');

  await arque.disconnect();
}

main();
