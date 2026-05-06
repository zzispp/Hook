import type {
  CleanupFn,
  ElementDragType,
  BaseEventPayload,
} from '@atlaskit/pragmatic-drag-and-drop/dist/types/internal-types';
import type {
  PublicConfig,
  ElementAutoScrollArgs,
} from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/dist/types/internal-types';
import type { IKanban } from 'src/types/kanban';

import { useRef, useEffect, useCallback } from 'react';
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine';
import { reorder } from '@atlaskit/pragmatic-drag-and-drop/reorder';
import { monitorForElements } from '@atlaskit/pragmatic-drag-and-drop/element/adapter';
import { extractClosestEdge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';
import { autoScrollForElements } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/element';
import { reorderWithEdge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/util/reorder-with-edge';
import { unsafeOverflowAutoScrollForElements } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/unsafe-overflow/element';

import { moveTask, moveColumn } from 'src/actions/kanban';

import { bindAll } from '../utils/bind-event-listener';
import { getAttr, triggerFlashEffect, isInvalidOrSameIndex } from '../utils/helpers';
import { isTaskData, isColumnData, isTaskDropTargetData } from '../utils/process-data';

// ----------------------------------------------------------------------

const BOARD_SCROLL_SPEED: PublicConfig['maxScrollSpeed'] = 'fast';
const BOARD_OVERFLOW_DISTANCE = 1000;
const PANNING_STOP_EVENTS = [
  'pointercancel',
  'pointerup',
  'pointerdown',
  'keydown',
  'resize',
  'click',
  'visibilitychange',
] as const;

export type UseBoardDndReturn = {
  boardRef: React.RefObject<HTMLDivElement | null>;
};

export function useBoardDnd(board: IKanban): UseBoardDndReturn {
  const boardRef = useRef<HTMLDivElement>(null);

  const handleTaskDrop = useCallback(
    ({ source, location }: BaseEventPayload<ElementDragType>) => {
      const dropTarget = location.current.dropTargets[0];
      if (!dropTarget || !isTaskData(source.data)) return;

      const sourceData = source.data;
      const targetData = dropTarget.data;
      const sourceColumnId = sourceData.columnId;

      const sourceTasks = board.tasks[sourceColumnId];
      if (!sourceTasks) return;

      const sourceTaskIndex = sourceTasks.findIndex((task) => task.id === sourceData.task.id);
      if (sourceTaskIndex === -1) return;

      // Task-to-task drops (reorder within column or move between columns)
      if (isTaskDropTargetData(targetData)) {
        const targetColumnId = targetData.columnId;

        // ➤ Same column: reorder tasks
        if (sourceColumnId === targetColumnId) {
          const targetTaskIndex = sourceTasks.findIndex((task) => task.id === targetData.task.id);
          if (isInvalidOrSameIndex(sourceTaskIndex, targetTaskIndex)) return;

          const reorderedTasks = reorderWithEdge({
            axis: 'vertical',
            list: sourceTasks,
            startIndex: sourceTaskIndex,
            indexOfTarget: targetTaskIndex,
            closestEdgeOfTarget: extractClosestEdge(targetData),
          });

          const sourceTaskId = sourceData.task.id;
          const newIndex = reorderedTasks.findIndex((task) => task.id === sourceTaskId);

          if (sourceTaskIndex !== newIndex) {
            moveTask({ ...board.tasks, [sourceColumnId]: reorderedTasks });
            triggerFlashEffect(getAttr('dataTaskId'), sourceData.task.id);
          }
          return;
        }

        // ➤ Different column: move task to new position
        const targetTasks = board.tasks[targetColumnId];
        if (!targetTasks) return;

        const targetTaskIndex = targetTasks.findIndex((task) => task.id === targetData.task.id);
        const closestEdge = extractClosestEdge(targetData);
        const insertIndex = closestEdge === 'bottom' ? targetTaskIndex + 1 : targetTaskIndex;

        // Remove from source column
        const updatedSourceTasks = [...sourceTasks];
        updatedSourceTasks.splice(sourceTaskIndex, 1);
        // Insert into target column at calculated position
        const updatedTargetTasks = [...targetTasks];
        updatedTargetTasks.splice(insertIndex, 0, sourceData.task);

        moveTask({
          ...board.tasks,
          [sourceColumnId]: updatedSourceTasks,
          [targetColumnId]: updatedTargetTasks,
        });
        triggerFlashEffect(getAttr('dataTaskId'), sourceData.task.id);
        return;
      }

      // ➤ Task dropped onto column (append to end)
      if (isColumnData(targetData)) {
        const targetColumnId = targetData.column.id;
        const targetTasks = board.tasks[targetColumnId];
        if (!targetTasks) return;

        // ➤ Same column: move to end
        if (sourceColumnId === targetColumnId) {
          const finalIndex = sourceTasks.length - 1;

          if (sourceTaskIndex !== finalIndex) {
            const reorderedTasks = reorder({
              list: sourceTasks,
              startIndex: sourceTaskIndex,
              finishIndex: finalIndex,
            });

            moveTask({ ...board.tasks, [sourceColumnId]: reorderedTasks });
            triggerFlashEffect(getAttr('dataTaskId'), sourceData.task.id);
          }
          return;
        }

        // ➤ Different column: append to end
        const updatedSourceTasks = [...sourceTasks];
        updatedSourceTasks.splice(sourceTaskIndex, 1);
        const updatedTargetTasks = [...targetTasks, sourceData.task];

        moveTask({
          ...board.tasks,
          [sourceColumnId]: updatedSourceTasks,
          [targetColumnId]: updatedTargetTasks,
        });
        triggerFlashEffect(getAttr('dataTaskId'), sourceData.task.id);
      }
    },
    [board.tasks]
  );

  const handleColumnDrop = useCallback(
    ({ source, location }: BaseEventPayload<ElementDragType>) => {
      const dropTarget = location.current.dropTargets[0];
      if (!dropTarget) return;

      const sourceData = source.data;
      const targetData = dropTarget.data;
      if (!isColumnData(sourceData) || !isColumnData(targetData)) return;

      const sourceIndex = board.columns.findIndex((column) => column.id === sourceData.column.id);
      const targetIndex = board.columns.findIndex((column) => column.id === targetData.column.id);

      if (isInvalidOrSameIndex(sourceIndex, targetIndex)) return;

      const reorderedColumns = reorder({
        list: board.columns,
        startIndex: sourceIndex,
        finishIndex: targetIndex,
      });

      moveColumn(reorderedColumns);
    },
    [board.columns]
  );

  useEffect(() => {
    const boardEl = boardRef.current;
    if (!boardEl) return undefined;

    /**
     * ➤➤ Task drag-and-drop monitoring
     */
    const taskMonitor = monitorForElements({
      canMonitor: ({ source }) => isTaskData(source.data),
      onDrop: handleTaskDrop,
    });

    /**
     * ➤➤ Column drag-and-drop monitoring
     */
    const columnMonitor = monitorForElements({
      canMonitor: ({ source }) => isColumnData(source.data),
      onDrop: handleColumnDrop,
    });

    /**
     * ➤➤ Auto-scroll configuration for smooth scrolling
     */
    const scrollConfig: ElementAutoScrollArgs<ElementDragType> = {
      element: boardEl,
      getConfiguration: () => ({ maxScrollSpeed: BOARD_SCROLL_SPEED }),
      canScroll: ({ source }) => isTaskData(source.data) || isColumnData(source.data),
    };

    const scrollBoard = autoScrollForElements(scrollConfig);

    /**
     * ➤➤ Overflow scroll for dragging beyond visible boundaries
     */
    const overflowBoardScroll = unsafeOverflowAutoScrollForElements({
      ...scrollConfig,
      getOverflow: () => ({
        forLeftEdge: {
          top: BOARD_OVERFLOW_DISTANCE,
          left: BOARD_OVERFLOW_DISTANCE,
          bottom: BOARD_OVERFLOW_DISTANCE,
        },
        forRightEdge: {
          top: BOARD_OVERFLOW_DISTANCE,
          right: BOARD_OVERFLOW_DISTANCE,
          bottom: BOARD_OVERFLOW_DISTANCE,
        },
      }),
    });

    return combine(taskMonitor, columnMonitor, scrollBoard, overflowBoardScroll);
  }, [board.tasks, handleTaskDrop, handleColumnDrop]);

  // Enable horizontal panning (click + drag to scroll the board)
  useHorizontalPanning(boardRef);

  return {
    boardRef,
  };
}

// ----------------------------------------------------------------------

/**
 * Enables horizontal board panning (click and drag to scroll horizontally).
 * Ignores interactive elements (e.g., buttons, inputs) with blocking attributes.
 */
function useHorizontalPanning(boardRef: React.RefObject<HTMLDivElement | null>) {
  useEffect(() => {
    const boardEl = boardRef.current;
    if (!boardEl) return undefined;

    let cleanupPanning: CleanupFn | null = null;

    // Start listening for pointer movement to scroll horizontally
    const handleStartPanning = (initialX: number) => {
      let lastPointerX = initialX;

      const stopEvents = PANNING_STOP_EVENTS.map((eventName) => ({
        type: eventName,
        listener: () => cleanup(),
      }));

      const cleanup = bindAll(
        window,
        [
          {
            type: 'pointermove',
            listener(event) {
              const currentPointerX = event.clientX;
              const deltaX = lastPointerX - currentPointerX;

              lastPointerX = currentPointerX;
              boardEl.scrollBy({ left: deltaX });
            },
          },
          ...stopEvents, // Stop panning on any of these events
        ],
        { capture: true } // Use capture to ensure we catch events before other handlers
      );

      cleanupPanning = cleanup;
    };

    // Listen for pointer down to initiate panning
    const cleanupPointerDown = bindAll(boardEl, [
      {
        type: 'pointerdown',
        listener(event) {
          const target = event.target as HTMLElement;
          if (!target) return;

          // Skip interactive elements (e.g., buttons, inputs) that should not trigger panning
          const isBlocked =
            target.hasAttribute(getAttr('blockBoardPanning')) ||
            target.closest(`[${getAttr('blockBoardPanning')}]`);

          if (isBlocked) return;

          handleStartPanning(event.clientX);
        },
      },
    ]);

    return () => {
      cleanupPointerDown();
      cleanupPanning?.();
    };
  }, [boardRef]);
}
