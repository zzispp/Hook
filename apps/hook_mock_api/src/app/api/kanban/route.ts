import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _board } from 'src/_mock/_kanban';

// ----------------------------------------------------------------------

export const runtime = 'edge';

type BoardType = ReturnType<typeof _board>;
let boardData: BoardType = _board();

const ENDPOINTS = {
  CREATE_COLUMN: 'create-column',
  UPDATE_COLUMN: 'update-column',
  MOVE_COLUMN: 'move-column',
  CLEAR_COLUMN: 'clear-column',
  DELETE_COLUMN: 'delete-column',
  CREATE_TASK: 'create-task',
  UPDATE_TASK: 'update-task',
  MOVE_TASK: 'move-task',
  DELETE_TASK: 'delete-task',
};

function loggerData(action?: string, value?: unknown) {
  const columnsWithTasks = boardData.columns.map(
    (col) => `${col.name} (${boardData.tasks[col.id].length} tasks)`
  );
  logger(
    '[Kanban] get-board',
    `columns (${boardData.columns.length}): ${JSON.stringify(columnsWithTasks, null, 2)}`
  );
  if (value || action) {
    logger(`[Kanban] ${action}`, value);
  }
}

function updateBoardData(newData: Partial<BoardType>) {
  boardData = { ...boardData, ...newData };
}

/** **************************************
 * GET - Board
 *************************************** */
export async function GET() {
  try {
    loggerData();

    return response({ board: boardData }, STATUS.OK);
  } catch (error) {
    return handleError('Kanban - Get board', error);
  }
}

/** **************************************
 * POST - Handle actions based on the endpoint
 *************************************** */
export async function POST(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const endpoint = searchParams.get('endpoint');

    switch (endpoint) {
      case ENDPOINTS.CREATE_COLUMN:
        return createColumn(req);
      case ENDPOINTS.UPDATE_COLUMN:
        return updateColumn(req);
      case ENDPOINTS.MOVE_COLUMN:
        return moveColumn(req);
      case ENDPOINTS.CLEAR_COLUMN:
        return clearColumn(req);
      case ENDPOINTS.DELETE_COLUMN:
        return deleteColumn(req);
      case ENDPOINTS.CREATE_TASK:
        return createTask(req);
      case ENDPOINTS.UPDATE_TASK:
        return updateTask(req);
      case ENDPOINTS.MOVE_TASK:
        return moveTask(req);
      case ENDPOINTS.DELETE_TASK:
        return deleteTask(req);
      default:
        return response({ message: 'Endpoint not found!' }, STATUS.NOT_FOUND);
    }
  } catch (error) {
    return handleError(`Kanban - Post request`, error);
  }
}

/** **************************************
 * COLUMN MANAGEMENT
 *************************************** */

/**
 * @Column Create
 * Create a new column in the board.
 */
async function createColumn(req: NextRequest) {
  const { columnData } = await req.json();

  // Add the new column and initialize its task list
  updateBoardData({
    columns: [...boardData.columns, columnData],
    tasks: { ...boardData.tasks, [columnData.id]: [] },
  });

  loggerData('created-column', columnData.name);

  return response({ column: columnData }, STATUS.OK);
}

/**
 * @Column Update
 * Update the name of an existing column.
 */
async function updateColumn(req: NextRequest) {
  const { columnId, columnName } = await req.json();

  const column = boardData.columns.find((col) => col.id === columnId);

  if (!column) {
    return response({ message: 'Column not found!' }, STATUS.NOT_FOUND);
  }

  // Find and update the specified column.
  updateBoardData({
    columns: boardData.columns.map((col) =>
      col.id === columnId ? { ...col, name: columnName } : col
    ),
  });

  loggerData('updated-column', columnName);
  return response({ columnId, columnName }, STATUS.OK);
}

/**
 * @Column Move
 * Reorder columns in the board.
 */
async function moveColumn(req: NextRequest) {
  const { updateColumns } = await req.json();

  // Update the column order
  updateBoardData({
    columns: updateColumns,
  });

  loggerData('moved-column', 'success!');

  return response({ columns: updateColumns }, STATUS.OK);
}

/**
 * @Column Clear
 * Remove all tasks from a specific column.
 */
async function clearColumn(req: NextRequest) {
  const { columnId } = await req.json();

  // Clear tasks for the specified column
  updateBoardData({
    tasks: { ...boardData.tasks, [columnId]: [] },
  });

  loggerData('cleared-column', 'success!');

  return response({ columnId }, STATUS.OK);
}

/**
 * @Column Delete
 * Delete a column and its associated tasks.
 */
async function deleteColumn(req: NextRequest) {
  const { columnId } = await req.json();

  // Remove the column and its tasks
  updateBoardData({
    columns: boardData.columns.filter((col) => col.id !== columnId),
    tasks: Object.fromEntries(Object.entries(boardData.tasks).filter(([id]) => id !== columnId)),
  });

  loggerData('deleted-column', columnId);

  return response({ columnId }, STATUS.OK);
}

/** **************************************
 * TASK MANAGEMENT
 *************************************** */

/**
 * @Task Create
 * Add a new task to a specific column.
 */
async function createTask(req: NextRequest) {
  const { columnId, taskData } = await req.json();

  // Add the new task to the specified column
  updateBoardData({
    tasks: {
      ...boardData.tasks,
      [columnId]: [taskData, ...boardData.tasks[columnId]],
    },
  });

  loggerData('created-task', taskData.name);

  return response({ columnId, taskData }, STATUS.OK);
}

/**
 * @Task Update
 * Update an existing task in a specific column.
 */
async function updateTask(req: NextRequest) {
  const { columnId, taskData } = await req.json();

  // Update the task in the specified column
  updateBoardData({
    tasks: {
      ...boardData.tasks,
      [columnId]: boardData.tasks[columnId].map((task) =>
        task.id === taskData.id ? { ...task, ...taskData } : task
      ),
    },
  });

  loggerData('updated-task', taskData.name);

  return response({ task: taskData }, STATUS.OK);
}

/**
 * @Task Move
 * Move a task between columns or reorder within the same column.
 */
async function moveTask(req: NextRequest) {
  const { updateTasks } = await req.json();

  // Update the task structure
  updateBoardData({
    tasks: updateTasks,
  });

  loggerData('moved-task', 'success!');

  return response({ tasks: updateTasks }, STATUS.OK);
}

/**
 * @Task Delete
 * Remove a task from a specific column.
 */
async function deleteTask(req: NextRequest) {
  const { columnId, taskId } = await req.json();

  // Remove the task from the specified column
  updateBoardData({
    tasks: {
      ...boardData.tasks,
      [columnId]: boardData.tasks[columnId].filter((task) => task.id !== taskId),
    },
  });

  loggerData('deleted-task', taskId);

  return response({ columnId, taskId }, STATUS.OK);
}
