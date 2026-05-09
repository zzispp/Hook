'use client';

import Stack from '@mui/material/Stack';

import { AddButton, RefreshButton } from './shared';

type RefreshAddActionsProps = {
  loading?: boolean;
  addLabel: string;
  onAdd: () => void;
  onRefresh: () => void;
};

export function RefreshAddActions({
  loading,
  addLabel,
  onAdd,
  onRefresh,
}: RefreshAddActionsProps) {
  return (
    <Stack direction="row" spacing={1}>
      <RefreshButton loading={loading} onClick={onRefresh} />
      <AddButton onClick={onAdd}>{addLabel}</AddButton>
    </Stack>
  );
}
