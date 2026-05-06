import { createClasses } from 'src/theme/create-classes';

// ----------------------------------------------------------------------

export const kanbanClasses = {
  column: {
    list: createClasses('kanban__column_list'),
    root: createClasses('kanban__column__root'),
    wrapper: createClasses('kanban__column__wrapper'),
  },
  item: {
    root: createClasses('kanban__item__root'),
  },
  state: {
    idle: '--idle',
    flash: '--flash',
    preview: '--preview',
    dragging: '--dragging',
    taskOver: '--task-over',
    columnOver: '--column-over',
    openDetails: '--open-details',
    draggingAndLeftSelf: '--dragging-and-left-self',
  },
} as const;
