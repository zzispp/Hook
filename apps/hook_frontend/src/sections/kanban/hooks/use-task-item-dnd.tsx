import type { Edge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';
import type { IKanbanTask } from 'src/types/kanban';

import { useRef, useState, useEffect } from 'react';
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine';
import {
  draggable,
  dropTargetForElements,
} from '@atlaskit/pragmatic-drag-and-drop/element/adapter';
import { preserveOffsetOnSource } from '@atlaskit/pragmatic-drag-and-drop/element/preserve-offset-on-source';
import {
  attachClosestEdge,
  extractClosestEdge,
} from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview';

import { kanbanClasses } from '../classes';
import { isShallowEqual } from '../utils/helpers';
import { isTaskData, getTaskData, getTaskDropTargetData } from '../utils/process-data';

// ----------------------------------------------------------------------

export type TaskState =
  | { type: typeof kanbanClasses.state.idle }
  | { type: typeof kanbanClasses.state.dragging }
  | { type: typeof kanbanClasses.state.draggingAndLeftSelf }
  | { type: typeof kanbanClasses.state.taskOver; dragRect: DOMRect; closestEdge: Edge }
  | { type: typeof kanbanClasses.state.preview; dragRect: DOMRect; container: HTMLElement };

export type UseTaskItemDndReturn = {
  state: TaskState;
  taskRef: React.RefObject<HTMLLIElement | null>;
};

export function useTaskItemDnd(task: IKanbanTask, columnId: string): UseTaskItemDndReturn {
  const taskRef = useRef<HTMLLIElement>(null);

  const [state, setState] = useState<TaskState>({ type: kanbanClasses.state.idle });

  useEffect(() => {
    const taskEl = taskRef.current;
    if (!taskEl) return undefined;

    /**
     * ➤➤ Makes the task draggable.
     *
     * (1) getInitialData => Provide initial drag data when the drag starts
     * (2) onDragStart => When dragging starts
     * (3) onDrop => When the item is dropped
     * (4) onGenerateDragPreview => Customize the drag preview behavior
     */
    const dragTask = draggable({
      element: taskEl,
      getInitialData: ({ element }) =>
        getTaskData({
          task,
          columnId,
          rect: element.getBoundingClientRect(),
        }),
      onDragStart: () => setState({ type: kanbanClasses.state.dragging }),
      onDrop: () => setState({ type: kanbanClasses.state.idle }),
      onGenerateDragPreview: ({ location, source, nativeSetDragImage }) => {
        if (!isTaskData(source.data)) return;

        setCustomNativeDragPreview({
          nativeSetDragImage,
          getOffset: preserveOffsetOnSource({
            element: taskEl,
            input: location.current.input,
          }),
          render: ({ container }) => {
            setState({
              type: kanbanClasses.state.preview,
              dragRect: taskEl.getBoundingClientRect(),
              container,
            });
            return () => setState({ type: kanbanClasses.state.dragging });
          },
        });
      },
    });

    /**
     * ➤➤ Registers the task as a drop target.
     *
     * (1) getIsSticky => Always keep active as a drop target
     * (2) canDrop => Can drop if source is a valid task
     * (3) getData => Provide data with closest edge information
     * (4) onDrag => While dragging over this item
     * (5) onDragEnter => When the dragged item enters this drop target
     * (6) onDragLeave => When the dragged item leaves this target
     * (7) onDrop => When the item is dropped
     */
    const dropTaskTarget = dropTargetForElements({
      element: taskEl,
      getIsSticky: () => true,
      canDrop: ({ source }) => isTaskData(source.data),
      getData: ({ input, element }) => {
        const userData = getTaskDropTargetData({ task, columnId });
        return attachClosestEdge(userData, {
          input,
          element,
          allowedEdges: ['top', 'bottom'],
        });
      },
      onDrag: ({ source, self }) => {
        if (!isTaskData(source.data) || source.data.task.id === task.id) return;

        const closestEdge = extractClosestEdge(self.data);
        if (!closestEdge) return;

        const nextState: TaskState = {
          type: kanbanClasses.state.taskOver,
          dragRect: source.data.rect,
          closestEdge,
        };

        setState((prevState) => (isShallowEqual(prevState, nextState) ? prevState : nextState));
      },
      onDragEnter: ({ source, self }) => {
        if (!isTaskData(source.data) || source.data.task.id === task.id) return;

        const closestEdge = extractClosestEdge(self.data);
        if (!closestEdge) return;

        setState({
          type: kanbanClasses.state.taskOver,
          dragRect: source.data.rect,
          closestEdge,
        });
      },
      onDragLeave: ({ source }) => {
        if (!isTaskData(source.data)) return;

        const isSelf = source.data.task.id === task.id;

        setState({
          type: isSelf ? kanbanClasses.state.draggingAndLeftSelf : kanbanClasses.state.idle,
        });
      },
      onDrop: () => setState({ type: kanbanClasses.state.idle }),
    });

    return combine(dragTask, dropTaskTarget);
  }, [task, columnId]);

  return {
    taskRef,
    state,
  };
}
