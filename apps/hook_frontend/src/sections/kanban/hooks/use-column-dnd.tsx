import type { DragLocationHistory } from '@atlaskit/pragmatic-drag-and-drop/dist/types/internal-types';
import type { PublicConfig } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/dist/types/internal-types';
import type { IKanbanColumn } from 'src/types/kanban';
import type { TaskData } from '../utils/process-data';

import { useRef, useState, useEffect } from 'react';
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine';
import { autoScrollForElements } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/element';
import {
  draggable,
  dropTargetForElements,
} from '@atlaskit/pragmatic-drag-and-drop/element/adapter';
import { preserveOffsetOnSource } from '@atlaskit/pragmatic-drag-and-drop/element/preserve-offset-on-source';
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview';
import { unsafeOverflowAutoScrollForElements } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/unsafe-overflow/element';

import { kanbanClasses } from '../classes';
import { isShallowEqual } from '../utils/helpers';
import {
  isTaskData,
  isColumnData,
  getColumnData,
  isTaskDropTargetData,
} from '../utils/process-data';

// ----------------------------------------------------------------------

const COLUMN_SCROLL_SPEED: PublicConfig['maxScrollSpeed'] = 'standard';
const COLUMN_OVERFLOW_DISTANCE = 1000;

export type ColumnState =
  | { type: typeof kanbanClasses.state.idle }
  | { type: typeof kanbanClasses.state.dragging }
  | { type: typeof kanbanClasses.state.columnOver }
  | { type: typeof kanbanClasses.state.taskOver; dragRect: DOMRect; isOverChildTask: boolean };

export type UseColumnDndReturn = {
  state: ColumnState;
  columnRef: React.RefObject<HTMLDivElement | null>;
  columnWrapperRef: React.RefObject<HTMLLIElement | null>;
  taskListRef: React.RefObject<HTMLUListElement | null>;
  dragHandleRef: React.RefObject<HTMLDivElement | null>;
};

export function useColumnDnd(column: IKanbanColumn): UseColumnDndReturn {
  const columnRef = useRef<HTMLDivElement>(null);
  const columnWrapperRef = useRef<HTMLLIElement>(null);
  const taskListRef = useRef<HTMLUListElement>(null);
  const dragHandleRef = useRef<HTMLDivElement>(null);

  const [state, setState] = useState<ColumnState>({ type: kanbanClasses.state.idle });

  useEffect(() => {
    const columnWrapperEl = columnWrapperRef.current;
    const columnEl = columnRef.current;
    const taskListEl = taskListRef.current;
    const dragHandleEl = dragHandleRef.current;
    if (!columnWrapperEl || !columnEl || !dragHandleEl || !taskListEl) return undefined;

    /**
     * ➤➤ Updates column state when a task is dragged over it.
     */
    const handleTaskOverState = (data: TaskData, location: DragLocationHistory) => {
      const dropTarget = location.current.dropTargets[0];

      const nextState: ColumnState = {
        dragRect: data.rect,
        type: kanbanClasses.state.taskOver,
        isOverChildTask: Boolean(dropTarget && isTaskDropTargetData(dropTarget.data)),
      };

      setState((prevState) => (isShallowEqual(prevState, nextState) ? prevState : nextState));
    };

    /**
     * ➤➤ Makes the column draggable using its handle.
     *
     * (1) getInitialData => Provide initial drag data when the drag starts
     * (2) onDragStart => When dragging starts
     * (3) onDrop => When the item is dropped
     * (4) onGenerateDragPreview => Customize the drag preview behavior
     */
    const dragColumn = draggable({
      element: columnEl,
      dragHandle: dragHandleEl,
      getInitialData: () => getColumnData({ column }),
      onDragStart: () => setState({ type: kanbanClasses.state.dragging }),
      onDrop: () => setState({ type: kanbanClasses.state.idle }),
      onGenerateDragPreview: ({ source, location, nativeSetDragImage }) => {
        if (!isColumnData(source.data)) return;

        setCustomNativeDragPreview({
          nativeSetDragImage,
          getOffset: preserveOffsetOnSource({
            element: columnEl,
            input: location.current.input,
          }),
          render: ({ container }) => {
            const rect = columnEl.getBoundingClientRect();
            const previewEl = columnEl.cloneNode(true);
            if (!(previewEl instanceof HTMLElement)) return;

            Object.assign(previewEl.style, {
              width: `${rect.width}px`,
              height: `${rect.height}px`,
            });

            container.appendChild(previewEl);
          },
        });
      },
    });

    /**
     * ➤➤ Registers the column as a drop target for tasks and other columns.
     *
     * (1) getIsSticky => Always keep active as a drop target
     * (2) getData => Provide data with closest edge information
     * (3) canDrop => Can drop if source is a valid column
     * (4) onDragStart => When dragging starts
     * (5) onDragEnter => When the dragged item enters this drop target
     * (6) onDropTargetChange => When the drop target within the column changes (e.g. hover another task)
     * (7) onDragLeave => When the dragged item leaves this target
     * (8) onDrop => When the item is dropped
     */
    const dropColumnTarget = dropTargetForElements({
      element: columnWrapperEl,
      getIsSticky: () => true,
      getData: () => getColumnData({ column }),
      canDrop: ({ source }) => isTaskData(source.data) || isColumnData(source.data),
      onDragStart: ({ source, location }) => {
        if (isTaskData(source.data)) {
          handleTaskOverState(source.data, location);
        }
      },
      onDragEnter: ({ source, location }) => {
        if (isTaskData(source.data)) {
          handleTaskOverState(source.data, location);
          return;
        }
        if (isColumnData(source.data) && source.data.column.id !== column.id) {
          setState({ type: kanbanClasses.state.columnOver });
        }
      },
      onDropTargetChange: ({ source, location }) => {
        if (isTaskData(source.data)) {
          handleTaskOverState(source.data, location);
          return;
        }
      },
      onDragLeave: ({ source }) => {
        if (isColumnData(source.data) && source.data.column.id === column.id) return;
        setState({ type: kanbanClasses.state.idle });
      },
      onDrop: () => setState({ type: kanbanClasses.state.idle }),
    });

    /**
     * ➤➤ Enables vertical auto-scroll inside the column’s task list.
     */
    const scrollTaskList = autoScrollForElements({
      element: taskListEl,
      canScroll: ({ source }) => isTaskData(source.data),
      getConfiguration: () => ({ maxScrollSpeed: COLUMN_SCROLL_SPEED }),
    });

    /**
     * ➤➤ Enables scroll overflow when dragging near top/bottom of the task list.
     */
    const overflowTaskListScroll = unsafeOverflowAutoScrollForElements({
      element: taskListEl,
      canScroll: ({ source }) => isTaskData(source.data),
      getConfiguration: () => ({ maxScrollSpeed: COLUMN_SCROLL_SPEED }),
      getOverflow: () => ({
        forTopEdge: { top: COLUMN_OVERFLOW_DISTANCE },
        forBottomEdge: { bottom: COLUMN_OVERFLOW_DISTANCE },
      }),
    });

    return combine(dragColumn, dropColumnTarget, scrollTaskList, overflowTaskListScroll);
  }, [column]);

  return {
    state,
    columnRef,
    taskListRef,
    columnWrapperRef,
    dragHandleRef,
  };
}
