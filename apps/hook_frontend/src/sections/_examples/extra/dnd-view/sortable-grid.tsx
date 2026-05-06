import type { MotionProps } from 'framer-motion';
import type {
  ElementDragType,
  BaseEventPayload,
} from '@atlaskit/pragmatic-drag-and-drop/dist/types/internal-types';
import type { ItemRootProps } from './components';
import type { DnDItem, ItemState } from './utils';
import type { SortableListProps } from './sortable-list';

import { m, AnimatePresence } from 'framer-motion';
import { mergeClasses } from 'minimal-shared/utils';
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine';
import { memo, useRef, useState, useEffect, useCallback } from 'react';
import { preserveOffsetOnSource } from '@atlaskit/pragmatic-drag-and-drop/element/preserve-offset-on-source';
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview';
import {
  draggable,
  monitorForElements,
  dropTargetForElements,
} from '@atlaskit/pragmatic-drag-and-drop/element/adapter';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';

import { dndClasses } from './classes';
import { useManageItems, isInvalidOrSameIndex } from './utils';
import { ItemRoot, AddButton, ItemActions, ListContainer, LayoutContainer } from './components';

// ----------------------------------------------------------------------

const GRID_ITEM_KEY = Symbol('grid-item');

type ItemData = {
  [GRID_ITEM_KEY]: true;
  item: DnDItem;
};

function getItemData(data: Omit<ItemData, typeof GRID_ITEM_KEY>): ItemData {
  return { [GRID_ITEM_KEY]: true, ...data };
}

function isItemData(value: Record<string | symbol, unknown>): value is ItemData {
  return Boolean(value[GRID_ITEM_KEY]);
}

function motionOptions(itemId: string): MotionProps {
  return {
    layout: 'position',
    layoutId: `grid-item-${itemId}`,
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.8 },
    transition: { layout: { type: 'spring', damping: 48, stiffness: 480 } },
  };
}

// ----------------------------------------------------------------------

export function SortableGrid({ sx, data, ...other }: SortableListProps) {
  const {
    addItem,
    removeItem,
    items: gridItems,
    setItems: setGridItems,
  } = useManageItems({ initialItems: data });

  const handleItemDrop = useCallback(
    ({ source, location }: BaseEventPayload<ElementDragType>) => {
      const dropTarget = location.current.dropTargets[0];
      if (!dropTarget) return;

      const sourceData = source.data;
      const targetData = dropTarget.data;
      if (!isItemData(sourceData) || !isItemData(targetData)) return;

      const sourceIndex = gridItems.findIndex((item) => item.id === sourceData.item.id);
      const targetIndex = gridItems.findIndex((item) => item.id === targetData.item.id);
      if (isInvalidOrSameIndex(sourceIndex, targetIndex)) return;

      const updatedItems = [...gridItems];
      [updatedItems[sourceIndex], updatedItems[targetIndex]] = [
        updatedItems[targetIndex],
        updatedItems[sourceIndex],
      ];

      setGridItems(updatedItems);
    },
    [gridItems, setGridItems]
  );

  useEffect(() => {
    const itemMonitor = monitorForElements({
      canMonitor: ({ source }) => isItemData(source.data),
      onDrop: handleItemDrop,
    });

    return itemMonitor;
  }, [handleItemDrop]);

  return (
    <LayoutContainer sx={sx} {...other}>
      <AddButton onClick={addItem} />
      <ListContainer layout="grid">
        <AnimatePresence mode="popLayout">
          {gridItems.map((item) => (
            <Box component={m.li} key={item.id} {...motionOptions(item.id)}>
              <SortableItem item={item} onDelete={() => removeItem(item.id)} />
            </Box>
          ))}
        </AnimatePresence>
      </ListContainer>
    </LayoutContainer>
  );
}

// ----------------------------------------------------------------------

type SortableItemProps = ItemRootProps & {
  item: DnDItem;
  onDelete: () => void;
};

const SortableItem = memo(({ sx, item, onDelete, ...other }: SortableItemProps) => {
  const theme = useTheme();

  const itemRef = useRef<HTMLDivElement>(null);
  const dragHandleRef = useRef<HTMLButtonElement>(null);

  const [state, setState] = useState<ItemState>({ type: dndClasses.state.idle });

  useEffect(() => {
    const itemEl = itemRef.current;
    const dragHandleEl = dragHandleRef.current;
    if (!itemEl || !dragHandleEl) return undefined;

    const dragItem = draggable({
      element: itemEl,
      dragHandle: dragHandleEl,
      getInitialData: () => getItemData({ item }),
      onDragStart: () => setState({ type: dndClasses.state.dragging }),
      onDrop: () => setState({ type: dndClasses.state.idle }),
      onGenerateDragPreview: ({ location, nativeSetDragImage }) => {
        setCustomNativeDragPreview({
          nativeSetDragImage,
          getOffset: preserveOffsetOnSource({
            element: dragHandleEl,
            input: location.current.input,
          }),
          render: ({ container }) => {
            const rect = itemEl.getBoundingClientRect();
            const previewEl = itemEl.cloneNode(true);
            if (!(previewEl instanceof HTMLElement)) return;

            Object.assign(previewEl.style, {
              width: `${rect.width}px`,
              height: `${rect.height}px`,
              backgroundColor: theme.vars.palette.background.paper,
            });

            container.appendChild(previewEl);
          },
        });
      },
    });

    const dropItemTarget = dropTargetForElements({
      element: itemEl,
      getData: () => getItemData({ item }),
      getIsSticky: () => true,
      canDrop: ({ source }) => source.element !== itemEl && isItemData(source.data),
      onDragEnter: () => setState({ type: dndClasses.state.over }),
      onDragLeave: () => setState({ type: dndClasses.state.idle }),
      onDrop: () => setState({ type: dndClasses.state.idle }),
    });

    return combine(dragItem, dropItemTarget);
  }, [item, theme.vars.palette.background.paper]);

  return (
    <ItemRoot
      ref={itemRef}
      className={mergeClasses([dndClasses.item], {
        [dndClasses.state.dragging]: state.type === dndClasses.state.dragging,
        [dndClasses.state.over]: state.type === dndClasses.state.over,
      })}
      sx={sx}
      {...other}
    >
      {item.name}
      <ItemActions dragHandleRef={dragHandleRef} onDelete={onDelete} />
    </ItemRoot>
  );
});
