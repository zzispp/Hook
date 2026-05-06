import type { IKanbanTask } from 'src/types/kanban';
import type { UseTaskItemDndReturn } from '../hooks/use-task-item-dnd';

import { toast } from 'sonner';
import { useCallback } from 'react';
import { createPortal } from 'react-dom';
import { useBoolean } from 'minimal-shared/hooks';
import { mergeClasses } from 'minimal-shared/utils';

import { deleteTask, updateTask } from 'src/actions/kanban';

import { kanbanClasses } from '../classes';
import { KanbanDetails } from '../details/kanban-details';
import { useTaskItemDnd } from '../hooks/use-task-item-dnd';
import { getAttr, isSafari, taskMotionOptions } from '../utils/helpers';
import {
  ItemRoot,
  ItemInfo,
  ItemName,
  ItemImage,
  ItemStatus,
  ItemContent,
  ItemPreview,
  DropIndicator,
} from './styles';

// ----------------------------------------------------------------------

const renderDropIndicator = (
  state: UseTaskItemDndReturn['state'],
  closestEdge: 'top' | 'bottom'
) =>
  state.type === kanbanClasses.state.taskOver && state.closestEdge === closestEdge ? (
    <DropIndicator sx={{ height: state.dragRect.height }} />
  ) : null;

const renderTaskPreview = (state: UseTaskItemDndReturn['state'], task: IKanbanTask) =>
  state.type === kanbanClasses.state.preview
    ? createPortal(
        <ItemPreview
          sx={{
            width: state.dragRect.width,
            ...(!isSafari() && { borderRadius: 'var(--kanban-item-radius)' }),
          }}
        >
          <ItemStatus status={task.priority} />
          <ItemName name={task.name} />
        </ItemPreview>,
        state.container
      )
    : null;

// ----------------------------------------------------------------------

type TaskItemProps = React.ComponentProps<typeof ItemRoot> & {
  task: IKanbanTask;
  columnId: string;
};

export function KanbanTaskItem({ task, columnId, sx, ...other }: TaskItemProps) {
  const taskDetailsDialog = useBoolean();
  const { taskRef, state } = useTaskItemDnd(task, columnId);

  const handleDeleteTask = useCallback(async () => {
    try {
      deleteTask(columnId, task.id);
      toast.success('Delete success!', { position: 'top-center' });
    } catch (error) {
      console.error(error);
    }
  }, [columnId, task.id]);

  const handleUpdateTask = useCallback(
    async (taskData: IKanbanTask) => {
      try {
        updateTask(columnId, taskData);
      } catch (error) {
        console.error(error);
      }
    },
    [columnId]
  );

  const renderTaskDetailsDialog = () => (
    <KanbanDetails
      task={task}
      open={taskDetailsDialog.value}
      onClose={taskDetailsDialog.onFalse}
      onUpdateTask={handleUpdateTask}
      onDeleteTask={handleDeleteTask}
    />
  );

  const renderTaskDisplay = () => (
    <ItemRoot
      ref={taskRef}
      {...taskMotionOptions(task.id)}
      {...{
        [getAttr('dataTaskId')]: task.id,
      }}
      className={mergeClasses([kanbanClasses.item.root], {
        [kanbanClasses.state.dragging]: state.type === kanbanClasses.state.dragging,
        [kanbanClasses.state.draggingAndLeftSelf]:
          state.type === kanbanClasses.state.draggingAndLeftSelf,
        [kanbanClasses.state.openDetails]: taskDetailsDialog.value,
      })}
      sx={sx}
      onClick={taskDetailsDialog.onTrue}
      {...other}
    >
      <ItemImage attachments={task.attachments} />
      <ItemContent>
        <ItemStatus status={task.priority} />
        <ItemName name={task.name} />
        <ItemInfo
          comments={task.comments}
          assignee={task.assignee}
          attachments={task.attachments}
        />
      </ItemContent>
    </ItemRoot>
  );

  return (
    <>
      {renderDropIndicator(state, 'top')}
      {renderTaskDisplay()}
      {renderDropIndicator(state, 'bottom')}
      {renderTaskPreview(state, task)}

      {renderTaskDetailsDialog()}
    </>
  );
}
