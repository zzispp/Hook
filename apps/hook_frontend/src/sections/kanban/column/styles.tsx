import type { CSSObject } from '@mui/material/styles';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import { styled } from '@mui/material/styles';

import { kanbanClasses } from '../classes';

// ----------------------------------------------------------------------

export const ColumnWrapper = styled(m.li)({
  flexShrink: 0,
  display: 'flex',
  userSelect: 'none',
  flexDirection: 'column',
  width: 'var(--kanban-column-width)',
});

export const ColumnRoot = styled('div')(({ theme }) => {
  const backgroundOverStyles: Record<'idle' | 'taskOver' | 'columnOver', CSSObject> = {
    idle: {
      '--background-over': varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
      ...theme.applyStyles('dark', {
        '--background-over': varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
      }),
      top: 0,
      left: 0,
      content: '""',
      width: '100%',
      height: '100%',
      borderWidth: '1px',
      position: 'absolute',
      pointerEvents: 'none',
      borderRadius: 'inherit',
      backgroundColor: 'transparent',
      transition: theme.transitions.create(['background-color']),
    },
    taskOver: {
      borderStyle: 'solid',
      backgroundColor: 'var(--background-over)',
      borderColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
    },
    columnOver: {
      borderStyle: 'dashed',
      backgroundColor: 'var(--background-over)',
      borderColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.24),
    },
  };

  return {
    display: 'flex',
    position: 'relative',
    flexDirection: 'column',
    gap: 'var(--kanban-item-gap)',
    borderRadius: 'var(--kanban-column-radius)',
    backgroundColor: theme.vars.palette.background.neutral,
    '&::before': backgroundOverStyles.idle,
    [`&.${kanbanClasses.state.dragging}`]: {
      opacity: 0.4,
    },
    [`&.${kanbanClasses.state.taskOver}`]: {
      '&::before': backgroundOverStyles.taskOver,
    },
    [`&.${kanbanClasses.state.columnOver}`]: {
      '&::before': backgroundOverStyles.columnOver,
      '& > *': {
        visibility: 'hidden',
      },
    },
  };
});

export const ColumnList = styled('ul')({
  minHeight: 80,
  display: 'flex',
  overflowY: 'auto',
  overflowAnchor: 'none',
  flexDirection: 'column',
  gap: 'var(--kanban-item-gap)',
  paddingLeft: 'var(--kanban-column-px)',
  paddingRight: 'var(--kanban-column-px)',
  paddingBottom: 'var(--kanban-column-pb)',
});
