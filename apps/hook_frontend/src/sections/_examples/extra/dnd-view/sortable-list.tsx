import type { Edge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';
import type {
  ElementDragType,
  BaseEventPayload,
} from '@atlaskit/pragmatic-drag-and-drop/dist/types/internal-types';
import type { ItemRootProps } from './components';
import type { DnDItem, ItemState } from './utils';

import { mergeClasses } from 'minimal-shared/utils';
import { flushSync, createPortal } from 'react-dom';
import { combine } from '@atlaskit/pragmatic-drag-and-drop/combine';
import { memo, useRef, useState, useEffect, useCallback } from 'react';
import { autoScrollForElements } from '@atlaskit/pragmatic-drag-and-drop-auto-scroll/element';
import { reorderWithEdge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/util/reorder-with-edge';
import { preserveOffsetOnSource } from '@atlaskit/pragmatic-drag-and-drop/element/preserve-offset-on-source';
import {
  attachClosestEdge,
  extractClosestEdge,
} from '@atlaskit/pragmatic-drag-and-drop-hitbox/closest-edge';
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview';
import {
  draggable,
  monitorForElements,
  dropTargetForElements,
} from '@atlaskit/pragmatic-drag-and-drop/element/adapter';

import Box from '@mui/material/Box';

import { dndClasses } from './classes';
import { isShallowEqual, useManageItems, triggerFlashEffect, isInvalidOrSameIndex } from './utils';
import {
  ItemRoot,
  AddButton,
  ItemActions,
  ItemPreview,
  ListContainer,
  LayoutContainer,
  DropIndicatorBox,
  DropIndicatorLine,
} from './components';

// ----------------------------------------------------------------------

const VERTICAL_ITEM_KEY = Symbol('vertical-item');
const HORIZONTAL_ITEM_KEY = Symbol('horizontal-item');

export type IndicatorShape = 'box' | 'line';
export type Orientation = 'vertical' | 'horizontal';

type ItemData =
  | { [VERTICAL_ITEM_KEY]: true; item: DnDItem; rect?: DOMRect }
  | { [HORIZONTAL_ITEM_KEY]: true; item: DnDItem; rect?: DOMRect };

const getAttr = (orientation: Orientation) => `data-${orientation}-item-id`;

function getItemData({
  rect,
  item,
  orientation,
}: Pick<ItemData, 'item' | 'rect'> & { orientation: Orientation }): ItemData {
  return orientation === 'vertical'
    ? { [VERTICAL_ITEM_KEY]: true, item, rect }
    : { [HORIZONTAL_ITEM_KEY]: true, item, rect };
}

function isItemData(
  value: Record<string | symbol, unknown>,
  orientation: Orientation
): value is ItemData {
  const key = orientation === 'vertical' ? VERTICAL_ITEM_KEY : HORIZONTAL_ITEM_KEY;
  return Boolean(value[key]);
}

// ----------------------------------------------------------------------

export type SortableListProps = React.ComponentProps<typeof LayoutContainer> & {
  data: DnDItem[];
  orientation?: Orientation;
  indicatorShape?: IndicatorShape;
};

export function SortableList({
  sx,
  data,
  indicatorShape = 'box',
  orientation = 'vertical',
  ...other
}: SortableListProps) {
  const listRef = useRef<HTMLUListElement>(null);
  const {
    addItem,
    removeItem,
    items: listItems,
    setItems: setListItems,
  } = useManageItems({ initialItems: data, sortableListRef: listRef, orientation });

  const handleItemDrop = useCallback(
    ({ source, location }: BaseEventPayload<ElementDragType>) => {
      const dropTarget = location.current.dropTargets[0];
      if (!dropTarget) return;

      const sourceData = source.data;
      const targetData = dropTarget.data;
      if (!isItemData(sourceData, orientation) || !isItemData(targetData, orientation)) return;

      const sourceIndex = listItems.findIndex((item) => item.id === sourceData.item.id);
      const targetIndex = listItems.findIndex((item) => item.id === targetData.item.id);
      if (isInvalidOrSameIndex(sourceIndex, targetIndex)) return;

      const reorderedItems = reorderWithEdge({
        axis: orientation,
        list: listItems,
        startIndex: sourceIndex,
        indexOfTarget: targetIndex,
        closestEdgeOfTarget: extractClosestEdge(targetData),
      });

      flushSync(() => {
        setListItems(reorderedItems);
      });

      const sourceItemId = sourceData.item.id;
      const newIndex = reorderedItems.findIndex((item) => item.id === sourceItemId);

      if (sourceIndex !== newIndex) {
        triggerFlashEffect(getAttr(orientation), sourceItemId);
      }
    },
    [listItems, setListItems, orientation]
  );

  useEffect(() => {
    const listEl = listRef.current;
    if (!listEl) return undefined;

    const itemMonitor = monitorForElements({
      canMonitor: ({ source }) => isItemData(source.data, orientation),
      onDrop: handleItemDrop,
    });

    const scrollList = autoScrollForElements({
      element: listEl,
      canScroll: ({ source }) => isItemData(source.data, orientation),
      getConfiguration: () => ({ maxScrollSpeed: 'fast' }),
    });

    return combine(itemMonitor, scrollList);
  }, [handleItemDrop, orientation]);

  return (
    <LayoutContainer sx={sx} {...other}>
      <AddButton onClick={addItem} />
      <ListContainer ref={listRef} layout={orientation}>
        {listItems.map((item) => (
          <SortableItem
            key={item.id}
            {...{
              [getAttr(orientation)]: item.id,
            }}
            item={item}
            orientation={orientation}
            indicatorShape={indicatorShape}
            onDelete={() => removeItem(item.id)}
            sx={{
              ...(orientation === 'vertical' && { py: 3 }),
              ...(orientation === 'horizontal' && { px: 8 }),
            }}
          />
        ))}
      </ListContainer>
    </LayoutContainer>
  );
}

// ----------------------------------------------------------------------

const renderPreview = (state: ItemState, item: DnDItem) =>
  state.type === dndClasses.state.preview && state.container
    ? createPortal(<ItemPreview>{item.name}</ItemPreview>, state.container)
    : null;

const renderDropIndicatorBox = (state: ItemState, closestEdge: Edge) =>
  state.type === dndClasses.state.over && state.closestEdge === closestEdge ? (
    <DropIndicatorBox
      sx={{
        ...(['left', 'right'].includes(closestEdge) && { width: state.dragRect?.width }),
        ...(['top', 'bottom'].includes(closestEdge) && { height: state.dragRect?.height }),
      }}
    />
  ) : null;

const renderDropIndicatorLine = (state: ItemState) =>
  state.type === dndClasses.state.over && state.closestEdge ? (
    <DropIndicatorLine edge={state.closestEdge} gap="var(--dnd-item-gap)" />
  ) : null;

type SortableItemProps = ItemRootProps & {
  item: DnDItem;
  orientation: Orientation;
  indicatorShape: IndicatorShape;
  onDelete: () => void;
};

const SortableItem = memo(
  ({ item, orientation, indicatorShape, onDelete, sx, ...other }: SortableItemProps) => {
    const itemRef = useRef<HTMLDivElement>(null);
    const dragHandleRef = useRef<HTMLButtonElement>(null);

    const [state, setState] = useState<ItemState>({ type: dndClasses.state.idle });

    const isVertical = orientation === 'vertical';

    useEffect(() => {
      const itemEl = itemRef.current;
      const dragHandleEl = dragHandleRef.current;
      if (!itemEl || !dragHandleEl) return undefined;

      const dragItem = draggable({
        element: itemEl,
        dragHandle: dragHandleEl,
        getInitialData: () =>
          getItemData({
            item,
            orientation,
            rect: itemEl.getBoundingClientRect(),
          }),
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
              setState({
                type: dndClasses.state.preview,
                dragRect: itemEl.getBoundingClientRect(),
                container,
              });
              return () => setState({ type: dndClasses.state.dragging });
            },
          });
        },
      });

      const dropItemTarget = dropTargetForElements({
        element: itemEl,
        getIsSticky: () => true,
        canDrop: ({ source }) =>
          orientation === 'vertical'
            ? isItemData(source.data, orientation)
            : isItemData(source.data, orientation) && source.element !== itemEl,
        getData: ({ input }) => {
          const userData = getItemData({ item, orientation });
          return attachClosestEdge(userData, {
            element: itemEl,
            input,
            allowedEdges: isVertical ? ['top', 'bottom'] : ['left', 'right'],
          });
        },
        onDrag: ({ source, self }) => {
          const sourceData = source.data;
          if (!isItemData(sourceData, orientation) || sourceData.item.id === item.id) return;

          const closestEdge = extractClosestEdge(self.data);
          if (!closestEdge) return;

          const nextState: ItemState = {
            type: dndClasses.state.over,
            dragRect: sourceData.rect,
            closestEdge,
          };

          setState((prevState) => (isShallowEqual(prevState, nextState) ? prevState : nextState));
        },
        onDragEnter: ({ source, self }) => {
          const sourceData = source.data;
          if (!isItemData(sourceData, orientation) || sourceData.item.id === item.id) return;

          const closestEdge = extractClosestEdge(self.data);
          if (!closestEdge) return;

          setState({
            type: dndClasses.state.over,
            dragRect: sourceData.rect,
            closestEdge,
          });
        },
        onDragLeave: ({ source }) => {
          if (!isItemData(source.data, orientation)) return;

          const isSelf = source.data.item.id === item.id;

          setState({
            type: isSelf ? dndClasses.state.draggingAndLeftSelf : dndClasses.state.idle,
          });
        },
        onDrop: () => setState({ type: dndClasses.state.idle }),
      });

      return combine(dragItem, dropItemTarget);
    }, [isVertical, item, orientation]);

    const itemProps: ItemRootProps = {
      ref: itemRef,
      className: mergeClasses([dndClasses.item], {
        [dndClasses.state.dragging]: state.type === dndClasses.state.dragging,
        [dndClasses.state.draggingAndLeftSelf]: state.type === dndClasses.state.draggingAndLeftSelf,
      }),
      sx,
      ...other,
    };

    const renderWithIndicatorBox = () => (
      <>
        {renderDropIndicatorBox(state, isVertical ? 'top' : 'left')}
        <ItemRoot as="li" {...itemProps}>
          {item.name}
          <ItemActions dragHandleRef={dragHandleRef} onDelete={onDelete} />
        </ItemRoot>
        {renderDropIndicatorBox(state, isVertical ? 'bottom' : 'right')}
      </>
    );

    const renderWithIndicatorLine = () => (
      <Box component="li" sx={{ position: 'relative' }}>
        <ItemRoot {...itemProps}>
          {item.name}
          <ItemActions dragHandleRef={dragHandleRef} onDelete={onDelete} />
        </ItemRoot>
        {renderDropIndicatorLine(state)}
      </Box>
    );

    return (
      <>
        {indicatorShape === 'box' ? renderWithIndicatorBox() : renderWithIndicatorLine()}
        {renderPreview(state, item)}
      </>
    );
  }
);
