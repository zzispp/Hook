'use client';

import type { CSSObject } from '@mui/material/styles';

import { useState } from 'react';
import { AnimatePresence } from 'framer-motion';

import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import { styled } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import GlobalStyles from '@mui/material/GlobalStyles';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useGetBoard } from 'src/actions/kanban';
import { DashboardContent } from 'src/layouts/dashboard';

import { EmptyContent } from 'src/components/empty-content';

import { kanbanClasses } from '../classes';
import { useBoardDnd } from '../hooks/use-board-dnd';
import { KanbanColumn } from '../column/kanban-column';
import { KanbanColumnAdd } from '../column/kanban-column-add';
import { KanbanColumnSkeleton } from '../components/kanban-skeleton';

// ----------------------------------------------------------------------

const inputGlobalStyles = () => (
  <GlobalStyles
    styles={{
      body: {
        '--kanban-item-gap': '16px',
        '--kanban-item-radius': '12px',
        '--kanban-column-gap': '24px',
        '--kanban-column-width': '336px',
        '--kanban-column-radius': '16px',
        '--kanban-column-pt': '20px',
        '--kanban-column-pb': '16px',
        '--kanban-column-px': '16px',
      },
    }}
  />
);

// ----------------------------------------------------------------------

export function KanbanView() {
  const { board, boardLoading, boardEmpty } = useGetBoard();
  const { boardRef } = useBoardDnd(board);

  const [columnFixed, setColumnFixed] = useState(false);

  const renderLoading = () => (
    <Box sx={{ gap: 'var(--kanban-column-gap)', display: 'flex', alignItems: 'flex-start' }}>
      <KanbanColumnSkeleton />
    </Box>
  );

  const renderEmpty = () => <EmptyContent filled sx={{ py: 10, maxHeight: { md: 480 } }} />;

  const renderList = () => (
    <FlexibleColumnContainer columnFixed={columnFixed}>
      <AnimatePresence>
        {board.columns.map((column) => (
          <KanbanColumn key={column.id} column={column} tasks={board.tasks[column.id]} />
        ))}
      </AnimatePresence>
      <KanbanColumnAdd />
    </FlexibleColumnContainer>
  );

  const renderHead = () => (
    <Box
      sx={{
        mb: 3,
        pr: { sm: 3 },
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}
    >
      <Typography variant="h4">Kanban</Typography>

      <FormControlLabel
        label="Fixed column"
        labelPlacement="start"
        control={
          <Switch
            checked={columnFixed}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
              setColumnFixed(event.target.checked);
            }}
            slotProps={{ input: { id: 'fixed-column-switch' } }}
          />
        }
      />
    </Box>
  );

  return (
    <>
      {inputGlobalStyles()}

      <DashboardContent
        maxWidth={false}
        sx={{
          pb: 0,
          pl: { sm: 3 },
          pr: { sm: 0 },
          minHeight: 0,
          flex: '1 1 0',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {renderHead()}

        <ScrollContainer ref={boardRef}>
          {boardLoading ? renderLoading() : <>{boardEmpty ? renderEmpty() : renderList()}</>}
        </ScrollContainer>
      </DashboardContent>
    </>
  );
}

// ----------------------------------------------------------------------

const flexStyles: CSSObject = {
  minHeight: 0,
  flex: '1 1 auto',
};

const ScrollContainer = styled('div')(({ theme }) => ({
  ...theme.mixins.scrollbarStyles(theme),
  ...flexStyles,
  display: 'flex',
  overflowX: 'auto',
  flexDirection: 'column',
}));

const FlexibleColumnContainer = styled('ul', {
  shouldForwardProp: (prop: string) => !['columnFixed', 'sx'].includes(prop),
})<{ columnFixed?: boolean }>(({ theme }) => ({
  display: 'flex',
  gap: 'var(--kanban-column-gap)',
  paddingTop: theme.spacing(2),
  paddingBottom: theme.spacing(2),
  variants: [
    {
      props: { columnFixed: true },
      style: {
        ...flexStyles,
        [`& .${kanbanClasses.column.root}`]: { ...flexStyles },
      },
    },
  ],
}));
