'use client';

import type { Theme } from '@mui/material/styles';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { RequestRecordJsonViewer } from './request-record-json-viewer';

type PayloadTabValue = 'requestHeaders' | 'requestBody' | 'responseBody';

type PayloadTab = {
  value: PayloadTabValue;
  label: string;
  emptyText: string;
  payload?: unknown | null;
};

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
  const [activeTab, setActiveTab] = useState<PayloadTabValue>('requestHeaders');
  const tabs: PayloadTab[] = [
    {
      value: 'requestHeaders',
      label: t('requestRecords.requestHeaders'),
      emptyText: t('requestRecords.noRequestHeaders'),
      payload: requestHeaders,
    },
    {
      value: 'requestBody',
      label: t('requestRecords.requestBody'),
      emptyText: t('requestRecords.noRequestBody'),
      payload: requestBody,
    },
    {
      value: 'responseBody',
      label: t('requestRecords.responseBody'),
      emptyText: t('requestRecords.noResponseBody'),
      payload: responseBody,
    },
  ];
  const activePanel = tabs.find((tab) => tab.value === activeTab);

  if (!activePanel) {
    throw new Error(`Unknown request record payload tab: ${activeTab}`);
  }

  return (
    <Stack spacing={2} sx={panelSx}>
      <Tabs
        value={activeTab}
        variant="scrollable"
        scrollButtons="auto"
        onChange={(_event, value: PayloadTabValue) => setActiveTab(value)}
        sx={tabsSx}
      >
        {tabs.map((tab) => (
          <Tab key={tab.value} value={tab.value} label={tab.label} sx={tabSx} />
        ))}
      </Tabs>
      <Divider />
      <Box role="tabpanel" aria-label={activePanel.label}>
        <PayloadContent value={activePanel.payload} emptyText={activePanel.emptyText} />
      </Box>
    </Stack>
  );
}

function PayloadContent({
  emptyText,
  value,
}: {
  emptyText: string;
  value?: unknown | null;
}) {
  const hasValue = hasPayload(value);

  if (!hasValue) {
    return (
      <Typography variant="body2" color="text.secondary">
        {emptyText}
      </Typography>
    );
  }

  return <RequestRecordJsonViewer value={value} />;
}

function hasPayload(value: unknown): boolean {
  if (value == null) return false;
  if (typeof value === 'string') return value.length > 0;
  if (Array.isArray(value)) return value.length > 0;
  if (typeof value === 'object') return Object.keys(value).length > 0;
  return true;
}

const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

const tabsSx = {
  minHeight: 40,
};

const tabSx = {
  minHeight: 40,
  px: 1.5,
};
