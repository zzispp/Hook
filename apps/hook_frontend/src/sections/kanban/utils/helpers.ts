import type { MotionProps } from 'framer-motion';
import type { IKanbanTask, IKanbanColumn } from 'src/types/kanban';

import { once } from '@atlaskit/pragmatic-drag-and-drop/once';

import { kanbanClasses } from '../classes';

// ----------------------------------------------------------------------

export const isSafari = once(
  (): boolean =>
    typeof navigator !== 'undefined' &&
    navigator.userAgent.includes('AppleWebKit') &&
    !navigator.userAgent.includes('Chrome')
);

// ----------------------------------------------------------------------

const attributeMap = {
  dataTaskId: 'data-task-id',
  dataColumnId: 'data-column-id',
  blockBoardPanning: 'data-block-board-panning',
} as const;

export type AttributeMap = typeof attributeMap;
export type AttributeKey = keyof AttributeMap;
export type AttributeValue = AttributeMap[AttributeKey];

export function getAttr<K extends AttributeKey>(key: K): AttributeMap[K] {
  return attributeMap[key];
}

// ----------------------------------------------------------------------

export type FlashEffectOptions = {
  duration?: number;
  className?: string;
};

export function triggerFlashEffect(
  attr: AttributeValue,
  targetId: IKanbanTask['id'] | IKanbanColumn['id'],
  options?: FlashEffectOptions
): void {
  const { duration = 1000, className = kanbanClasses.state.flash } = options ?? {};

  requestAnimationFrame(() => {
    const targetEl = document.querySelector(`[${attr}="${targetId}"]`);
    if (!targetEl || !(targetEl instanceof HTMLElement)) return;

    targetEl.classList.remove(className);
    targetEl.classList.add(className);

    setTimeout(() => {
      targetEl.classList.remove(className);
    }, duration);
  });
}

// ----------------------------------------------------------------------

export function isInvalidOrSameIndex(fromIndex: number, toIndex: number): boolean {
  return fromIndex < 0 || toIndex < 0 || fromIndex === toIndex;
}

export function isShallowEqual<T extends Record<string, unknown>>(a: T, b: T): boolean {
  const aKeys = Object.keys(a) as (keyof T)[];
  const bKeys = Object.keys(b) as (keyof T)[];

  if (aKeys.length !== bKeys.length) return false;

  return aKeys.every((key) => Object.is(a[key], b[key]));
}

// ----------------------------------------------------------------------

export const columnMotionOptions = (columnId: IKanbanColumn['id']): MotionProps => ({
  layout: 'position',
  layoutId: `kanban-column-${columnId}`,
  transition: {
    layout: { type: 'spring', damping: 48, stiffness: 480 },
  },
});

export const taskMotionOptions = (taskId: IKanbanTask['id']): MotionProps => ({
  layout: 'position',
  layoutId: `kanban-item-${taskId}`,
  transition: {
    layout: { type: 'spring', damping: 40, stiffness: 400 },
  },
});
