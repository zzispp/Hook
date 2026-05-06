import type { IKanbanTask, IKanbanColumn } from 'src/types/kanban';

// ----------------------------------------------------------------------

export const TASK_KEY = Symbol('task');
export const TASK_DROP_TARGET_KEY = Symbol('task-drop-target');
export const COLUMN_KEY = Symbol('column');

// Task data
export type TaskData = {
  [TASK_KEY]: true;
  task: IKanbanTask;
  columnId: string;
  rect: DOMRect;
};

export function getTaskData(data: Omit<TaskData, typeof TASK_KEY>): TaskData {
  return { [TASK_KEY]: true, ...data };
}

export function isTaskData(value: Record<string | symbol, unknown>): value is TaskData {
  return Boolean(value[TASK_KEY]);
}

// Drop target data for tasks
export type TaskDropTargetData = {
  [TASK_DROP_TARGET_KEY]: true;
  task: IKanbanTask;
  columnId: string;
};

export function getTaskDropTargetData(
  data: Omit<TaskDropTargetData, typeof TASK_DROP_TARGET_KEY>
): TaskDropTargetData {
  return { [TASK_DROP_TARGET_KEY]: true, ...data };
}

export function isTaskDropTargetData(
  value: Record<string | symbol, unknown>
): value is TaskDropTargetData {
  return Boolean(value[TASK_DROP_TARGET_KEY]);
}

// Column data
export type ColumnData = {
  [COLUMN_KEY]: true;
  column: IKanbanColumn;
};

export function getColumnData(data: Omit<ColumnData, typeof COLUMN_KEY>): ColumnData {
  return { [COLUMN_KEY]: true, ...data };
}

export function isColumnData(value: Record<string | symbol, unknown>): value is ColumnData {
  return Boolean(value[COLUMN_KEY]);
}
