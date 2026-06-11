'use client';

import type { ReactNode } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

type Props = {
  title: string;
  actionLabel: string;
  children: ReactNode;
  onAdd: () => void;
};

export function ProviderPanelSection({ title, actionLabel, children, onAdd }: Props) {
  return (
    <Box sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1 }}>
      <Stack
        direction="row"
        alignItems="center"
        justifyContent="space-between"
        sx={{ px: 2, py: 1.5 }}
      >
        <Typography variant="subtitle2">{title}</Typography>
        <Button
          color="inherit"
          variant="contained"
          startIcon={<Iconify icon="mingcute:add-line" />}
          onClick={onAdd}
        >
          {actionLabel}
        </Button>
      </Stack>
      <Divider />
      <Box sx={{ px: 2, py: 1 }}>{children}</Box>
    </Box>
  );
}
