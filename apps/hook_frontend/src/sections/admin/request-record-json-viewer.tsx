'use client';

import type { Theme } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';
import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Collapse from '@mui/material/Collapse';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import ButtonBase from '@mui/material/ButtonBase';

import { Iconify } from 'src/components/iconify';

const JSON_INDENT_SIZE = 2;
const PAYLOAD_MAX_HEIGHT = 420;

export function RequestRecordJsonViewer({ value }: { value: unknown }) {
  const [expanded, setExpanded] = useState(false);
  const formatted = useMemo(() => formatPayload(value), [value]);
  const summary = useMemo(() => payloadSummary(value), [value]);
  const canCollapse = isJsonContainer(value);

  useEffect(() => {
    setExpanded(false);
  }, [value]);

  if (!canCollapse) {
    return <PayloadCodeBlock value={formatted} />;
  }

  return (
    <Stack sx={viewerSx}>
      <ButtonBase onClick={() => setExpanded((current) => !current)} sx={summaryButtonSx}>
        <Iconify
          width={16}
          icon={expanded ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'}
        />
        <Typography variant="caption" sx={summaryTextSx}>
          {summary}
        </Typography>
      </ButtonBase>
      <Collapse in={expanded} timeout="auto" unmountOnExit>
        <PayloadCodeBlock value={formatted} />
      </Collapse>
    </Stack>
  );
}

function PayloadCodeBlock({ value }: { value: string }) {
  const theme = useTheme();

  return (
    <Box
      component="pre"
      sx={{
        m: 0,
        p: 1.5,
        maxHeight: PAYLOAD_MAX_HEIGHT,
        overflow: 'auto',
        typography: 'caption',
        fontFamily: 'monospace',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
      }}
    >
      {value}
    </Box>
  );
}

function isJsonContainer(value: unknown): boolean {
  return Array.isArray(value) || (value !== null && typeof value === 'object');
}

function payloadSummary(value: unknown): string {
  if (Array.isArray(value)) return `[ ... ] · ${value.length}`;
  if (value !== null && typeof value === 'object') return `{ ... } · ${Object.keys(value).length}`;
  return String(value);
}

function formatPayload(value: unknown): string {
  if (typeof value === 'string') return value;
  const formatted = JSON.stringify(value, null, JSON_INDENT_SIZE);
  if (typeof formatted === 'string') return formatted;
  return String(value);
}

const viewerSx = {
  overflow: 'hidden',
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

const summaryButtonSx = {
  gap: 1,
  px: 1.5,
  py: 1,
  width: 1,
  minHeight: 40,
  justifyContent: 'flex-start',
  bgcolor: 'background.neutral',
};

const summaryTextSx = {
  minWidth: 0,
  fontFamily: 'monospace',
  color: 'text.secondary',
  overflow: 'hidden',
  whiteSpace: 'nowrap',
  textOverflow: 'ellipsis',
};
