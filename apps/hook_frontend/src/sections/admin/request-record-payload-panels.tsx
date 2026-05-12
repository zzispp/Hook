'use client';

import type { Theme } from '@mui/material/styles';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

export function RequestRecordPayloadPanels({
  requestHeaders,
  requestBody,
  responseBody,
}: {
  requestHeaders?: unknown | null;
  requestBody?: unknown | null;
  responseBody?: unknown | null;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <PayloadPanel
        title={t('requestRecords.requestHeaders')}
        emptyText={t('requestRecords.noRequestHeaders')}
        value={requestHeaders}
      />
      <PayloadPanel
        title={t('requestRecords.requestBody')}
        emptyText={t('requestRecords.noRequestBody')}
        value={requestBody}
      />
      <PayloadPanel
        title={t('requestRecords.responseBody')}
        emptyText={t('requestRecords.noResponseBody')}
        value={responseBody}
      />
    </>
  );
}

function PayloadPanel({
  title,
  emptyText,
  value,
}: {
  title: string;
  emptyText: string;
  value?: unknown | null;
}) {
  const hasValue = hasPayload(value);

  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Typography variant="subtitle2">{title}</Typography>
      <Typography
        variant="body2"
        color="text.secondary"
        sx={{ whiteSpace: 'pre-wrap', fontFamily: hasValue ? 'monospace' : undefined }}
      >
        {hasValue ? formatPayload(value) : emptyText}
      </Typography>
    </Stack>
  );
}

function hasPayload(value: unknown): boolean {
  if (value == null) return false;
  if (typeof value === 'string') return value.length > 0;
  if (Array.isArray(value)) return value.length > 0;
  if (typeof value === 'object') return Object.keys(value).length > 0;
  return true;
}

function formatPayload(value: unknown): string {
  if (typeof value === 'string') return value;
  return JSON.stringify(value, null, 2);
}

const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};
