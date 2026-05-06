import type { BoxProps } from '@mui/material/Box';
import type { CSSObject } from '@mui/material/styles';
import type { ButtonProps } from '@mui/material/Button';
import type { IconButtonProps } from '@mui/material/IconButton';
import type { Edge } from '@atlaskit/pragmatic-drag-and-drop-hitbox/dist/types/types';
import type { Orientation } from './sortable-list';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import { styled } from '@mui/material/styles';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/components/iconify';

import { dndClasses } from './classes';

// ----------------------------------------------------------------------

/* **********************************************************************
 * Layout elements
 * **********************************************************************/
export function AddButton({ sx, ...other }: ButtonProps) {
  return (
    <Button
      variant="contained"
      sx={[{ alignSelf: 'flex-end' }, ...(Array.isArray(sx) ? sx : [sx])]}
      {...other}
    >
      + Add item
    </Button>
  );
}

export const LayoutContainer = styled('div')(({ theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  gap: theme.spacing(3),
}));

export const ListContainer = styled('ul', {
  shouldForwardProp: (prop: string) => !['layout', 'sx'].includes(prop),
})<{ layout?: 'grid' | Orientation }>(({ theme }) => ({
  ...theme.mixins.scrollbarStyles(theme),
  width: '100%',
  gap: 'var(--dnd-item-gap)',
  variants: [
    {
      props: (props) => props.layout === 'grid',
      style: {
        display: 'grid',
        gridTemplateColumns: 'repeat(2, 1fr)',
        [theme.breakpoints.up('sm')]: {
          gridTemplateColumns: 'repeat(4, 1fr)',
        },
      },
    },
    {
      props: (props) => props.layout === 'vertical' || props.layout === 'horizontal',
      style: {
        width: '100%',
        display: 'flex',
        padding: 'var(--dnd-item-gap)',
        borderRadius: 'var(--dnd-item-radius)',
        border: `dashed 1px ${theme.vars.palette.shared.paperOutlined}`,
      },
    },
    {
      props: (props) => props.layout === 'vertical',
      style: {
        height: 560,
        maxWidth: 480,
        overflowY: 'auto',
        alignSelf: 'center',
        flexDirection: 'column',
      },
    },
    {
      props: (props) => props.layout === 'horizontal',
      style: {
        overflowX: 'auto',
      },
    },
  ],
}));

/* **********************************************************************
 * Item elements
 * **********************************************************************/
export type ItemActionProps = BoxProps & {
  onDelete?: () => void;
  dragHandleRef: React.RefObject<HTMLButtonElement | null>;
};

export function ItemActions({ onDelete, dragHandleRef, sx, ...other }: ItemActionProps) {
  const buttonProps: IconButtonProps = {
    size: 'small',
    disableRipple: true,
    disableFocusRipple: true,
    disableTouchRipple: true,
  };

  return (
    <Box
      sx={[
        (theme) => ({
          p: 1,
          top: 0,
          right: 0,
          zIndex: 9,
          display: 'flex',
          position: 'absolute',
          [`& .${dndClasses.removeBtn}`]: {
            opacity: 0,
            transition: theme.transitions.create(['opacity']),
          },
          '&:hover': {
            [`& .${dndClasses.removeBtn}`]: { opacity: 0.48 },
          },
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <IconButton {...buttonProps} onClick={onDelete} className={dndClasses.removeBtn}>
        <Iconify icon="solar:close-circle-bold" />
      </IconButton>
      <IconButton {...buttonProps} ref={dragHandleRef}>
        <Iconify icon="custom:drag-dots-fill" />
      </IconButton>
    </Box>
  );
}

export const ItemPreview = styled('div')(({ theme }) => ({
  ...theme.typography.h6,
  minWidth: 80,
  textAlign: 'center',
  padding: theme.spacing(2),
  borderRadius: 'var(--dnd-item-radius)',
  backgroundColor: theme.vars.palette.background.paper,
}));

export type ItemRootProps = React.ComponentProps<typeof ItemRoot>;

export const ItemRoot = styled('div')(({ theme }) => {
  const transitionKey = 'moveFlash';

  const defaultStyles: CSSObject = {
    transform: 'scale(1)',
    color: varAlpha(theme.vars.palette.text.secondaryChannel, 0.24),
    borderColor: theme.vars.palette.shared.paperOutlined,
    backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.04),
  };

  const transitionStyles: CSSObject = {
    transform: 'scale(0.98)',
    color: theme.vars.palette.primary.main,
    borderColor: varAlpha(theme.vars.palette.primary.mainChannel, 0.24),
    backgroundColor: varAlpha(theme.vars.palette.primary.mainChannel, 0.24),
  };

  return {
    [`@keyframes ${transitionKey}`]: {
      from: transitionStyles,
      to: defaultStyles,
    },
    ...defaultStyles,
    ...theme.typography.h2,
    flexShrink: 0,
    borderWidth: 1,
    display: 'flex',
    textAlign: 'center',
    borderStyle: 'solid',
    alignItems: 'center',
    position: 'relative',
    justifyContent: 'center',
    padding: theme.spacing(5),
    borderRadius: 'var(--dnd-item-radius)',
    transition: theme.transitions.create(['color', 'border-color', 'background-color']),
    '&:hover': {
      color: theme.vars.palette.text.secondary,
    },
    [`&.${dndClasses.state.dragging}`]: {
      color: theme.vars.palette.text.secondary,
    },
    [`&.${dndClasses.state.over}`]: {
      color: theme.vars.palette.text.primary,
      backgroundColor: theme.vars.palette.action.selected,
    },
    [`&.${dndClasses.state.draggingAndLeftSelf}`]: {
      display: 'none',
    },
    [`&.${dndClasses.state.flash}`]: {
      animation: `${transitionKey} 320ms ease-in-out`,
    },
  };
});

/* **********************************************************************
 * Drop indicator
 * **********************************************************************/
export const DropIndicatorBox = styled('span')(({ theme }) => ({
  flexShrink: 0,
  borderRadius: 'var(--dnd-item-radius)',
  backgroundColor: theme.vars.palette.action.selected,
  border: `1px dashed ${varAlpha(theme.vars.palette.grey['500Channel'], 0.24)}`,
}));

export const DropIndicatorLine = styled('span', {
  shouldForwardProp: (prop: string) => !['edge', 'gap', 'sx'].includes(prop),
})<{ edge: Edge; gap: string }>(({ theme, gap }) => {
  const LINE_SIZE = 2;
  const CIRCLE_SIZE = 8;

  const cssVars: CSSObject = {
    '--line-thickness': `${LINE_SIZE}px`,
    '--line-offset': `calc(-0.5 * (${gap} + ${LINE_SIZE}px))`,
    '--circle-size': `${CIRCLE_SIZE}px`,
    '--circle-radius': `${CIRCLE_SIZE / 2}px`,
    '--negative-circle-size': `-${CIRCLE_SIZE}px`,
    '--offset-circle': `${(LINE_SIZE - CIRCLE_SIZE) / 2}px`,
  };

  return {
    ...cssVars,
    zIndex: 9,
    position: 'absolute',
    pointerEvents: 'none',
    backgroundColor: 'currentColor',
    color: theme.vars.palette.primary.light,
    '&::before': {
      content: '""',
      position: 'absolute',
      borderRadius: '9999px',
      width: 'var(--circle-size)',
      height: 'var(--circle-size)',
      border: 'var(--line-thickness) solid currentColor',
    },
    variants: [
      {
        props: (props) => ['top', 'bottom'].includes(props.edge),
        style: {
          right: 0,
          left: 'var(--circle-radius)',
          height: 'var(--line-thickness)',
          '&::before': { left: 'var(--negative-circle-size)' },
        },
      },
      {
        props: (props) => ['left', 'right'].includes(props.edge),
        style: {
          bottom: 0,
          top: 'var(--circle-radius)',
          width: 'var(--line-thickness)',
          '&::before': { top: 'var(--negative-circle-size)' },
        },
      },
      {
        props: (props) => props.edge === 'top',
        style: {
          top: 'var(--line-offset)',
          '&::before': { top: 'var(--offset-circle)' },
        },
      },
      {
        props: (props) => props.edge === 'right',
        style: {
          right: 'var(--line-offset)',
          '&::before': { right: 'var(--offset-circle)' },
        },
      },
      {
        props: (props) => props.edge === 'bottom',
        style: {
          bottom: 'var(--line-offset)',
          '&::before': { bottom: 'var(--offset-circle)' },
        },
      },
      {
        props: (props) => props.edge === 'left',
        style: {
          left: 'var(--line-offset)',
          '&::before': { left: 'var(--offset-circle)' },
        },
      },
    ],
  };
});
