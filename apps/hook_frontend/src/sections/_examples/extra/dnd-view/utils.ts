import type React from 'react';
import type { Edge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';

import { uuidv4 } from 'minimal-shared/utils';
import { useState, useCallback } from 'react';

import { dndClasses } from './classes';

// ----------------------------------------------------------------------

export type DnDItem = { id: string; name: string };

export type ItemState =
  | { type: typeof dndClasses.state.idle }
  | { type: typeof dndClasses.state.dragging }
  | { type: typeof dndClasses.state.draggingAndLeftSelf }
  | { type: typeof dndClasses.state.over; dragRect?: DOMRect; closestEdge?: Edge }
  | { type: typeof dndClasses.state.preview; dragRect?: DOMRect; container?: HTMLElement };

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

export type FlashEffectOptions = {
  duration?: number;
  className?: string;
};

export function triggerFlashEffect(
  attr: string,
  targetId: DnDItem['id'],
  options?: FlashEffectOptions
) {
  const { duration = 1000, className = dndClasses.state.flash } = options ?? {};

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

type UseManageItemsReturn = {
  items: DnDItem[];
  addItem: () => void;
  removeItem: (itemId: DnDItem['id']) => void;
  setItems: React.Dispatch<React.SetStateAction<DnDItem[]>>;
};

type UseManageItemsProps<T extends HTMLElement> = {
  initialItems: DnDItem[];
  sortableListRef?: React.RefObject<T | null>;
  orientation?: 'vertical' | 'horizontal';
};

export function useManageItems<T extends HTMLElement>({
  orientation,
  initialItems,
  sortableListRef,
}: UseManageItemsProps<T>): UseManageItemsReturn {
  const [items, setItems] = useState<DnDItem[]>(initialItems);

  const scrollToEnd = useCallback(() => {
    const sortableListEl = sortableListRef?.current;
    if (!sortableListEl || !orientation) return;

    const scrollOptions: ScrollToOptions =
      orientation === 'vertical'
        ? { top: sortableListEl.scrollHeight }
        : { left: sortableListEl.scrollWidth };

    setTimeout(() => {
      sortableListEl.scrollTo({ ...scrollOptions, behavior: 'smooth' });
    }, 0);
  }, [sortableListRef, orientation]);

  const addItem = useCallback(() => {
    setItems((prevItems) => {
      const newItem = {
        id: `id-${uuidv4()}`,
        name: `${prevItems.length + 1}`,
      };
      return [...prevItems, newItem];
    });

    scrollToEnd();
  }, [scrollToEnd]);

  const removeItem = useCallback((itemId: DnDItem['id']) => {
    setItems((prevItems) => prevItems.filter((item) => item.id !== itemId));
  }, []);

  return {
    items,
    addItem,
    setItems,
    removeItem,
  };
}
