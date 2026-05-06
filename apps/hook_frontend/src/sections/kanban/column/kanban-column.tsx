import type { IKanbanTask, IKanbanColumn } from 'src/types/kanban';

import { toast } from 'sonner';
import { memo, useCallback } from 'react';
import { AnimatePresence } from 'framer-motion';
import { useBoolean } from 'minimal-shared/hooks';
import { mergeClasses } from 'minimal-shared/utils';

import { createTask, clearColumn, updateColumn, deleteColumn } from 'src/actions/kanban';

import { kanbanClasses } from '../classes';
import { DropIndicator } from '../item/styles';
import { useColumnDnd } from '../hooks/use-column-dnd';
import { KanbanTaskItem } from '../item/kanban-task-item';
import { KanbanColumnToolBar } from './kanban-column-toolbar';
import { KanbanTaskAdd } from '../components/kanban-task-add';
import { getAttr, columnMotionOptions } from '../utils/helpers';
import { ColumnRoot, ColumnList, ColumnWrapper } from './styles';

// ----------------------------------------------------------------------

type ColumnProps = React.ComponentProps<typeof ColumnRoot> & {
  column: IKanbanColumn;
  tasks: IKanbanTask[];
};

const TaskList = memo(({ column, tasks }: ColumnProps) =>
  tasks.map((task) => <KanbanTaskItem key={task.id} task={task} columnId={column.id} />)
);

// ----------------------------------------------------------------------

export function KanbanColumn({ column, tasks, sx, ...other }: ColumnProps) {
  const { taskListRef, dragHandleRef, columnRef, columnWrapperRef, state } = useColumnDnd(column);

  const openAddTask = useBoolean();

  const handleUpdateColumn = useCallback(
    async (columnName: string) => {
      try {
        if (column.name !== columnName) {
          updateColumn(column.id, columnName);

          toast.success('Update success!', { position: 'top-center' });
        }
      } catch (error) {
        console.error(error);
      }
    },
    [column.id, column.name]
  );

  const handleClearColumn = useCallback(async () => {
    try {
      clearColumn(column.id);
    } catch (error) {
      console.error(error);
    }
  }, [column.id]);

  const handleDeleteColumn = useCallback(async () => {
    try {
      deleteColumn(column.id);

      toast.success('Delete success!', { position: 'top-center' });
    } catch (error) {
      console.error(error);
    }
  }, [column.id]);

  const handleAddTask = useCallback(
    async (taskData: IKanbanTask) => {
      try {
        createTask(column.id, taskData);

        openAddTask.onFalse();
      } catch (error) {
        console.error(error);
      }
    },
    [column.id, openAddTask]
  );

  const renderHeader = () => (
    <KanbanColumnToolBar
      dragHandleRef={dragHandleRef}
      totalTasks={tasks.length}
      columnName={column.name}
      onUpdateColumn={handleUpdateColumn}
      onClearColumn={handleClearColumn}
      onDeleteColumn={handleDeleteColumn}
      onToggleAddTask={openAddTask.onToggle}
    />
  );

  const renderAddTaskBox = () => (
    <KanbanTaskAdd
      status={column.name}
      openAddTask={openAddTask.value}
      onAddTask={handleAddTask}
      onCloseAddTask={openAddTask.onFalse}
    />
  );

  const renderDropIndicator = () =>
    state.type === kanbanClasses.state.taskOver && !state.isOverChildTask ? (
      <DropIndicator sx={{ height: state.dragRect.height }} />
    ) : null;

  const renderTaskList = () => (
    <ColumnList ref={taskListRef} className={kanbanClasses.column.list}>
      <AnimatePresence>
        <TaskList column={column} tasks={tasks} />
      </AnimatePresence>
      {renderDropIndicator()}
    </ColumnList>
  );

  return (
    <ColumnWrapper
      {...columnMotionOptions(column.id)}
      {...{
        [getAttr('dataColumnId')]: column.id,
      }}
      ref={columnWrapperRef}
      className={kanbanClasses.column.wrapper}
    >
      <ColumnRoot
        {...{
          [getAttr('blockBoardPanning')]: true,
        }}
        ref={columnRef}
        sx={sx}
        {...other}
        className={mergeClasses([kanbanClasses.column.root], {
          [kanbanClasses.state.dragging]: state.type === kanbanClasses.state.dragging,
          [kanbanClasses.state.taskOver]: state.type === kanbanClasses.state.taskOver,
          [kanbanClasses.state.columnOver]: state.type === kanbanClasses.state.columnOver,
        })}
      >
        {renderHeader()}
        {renderAddTaskBox()}
        {renderTaskList()}
      </ColumnRoot>
    </ColumnWrapper>
  );
}
