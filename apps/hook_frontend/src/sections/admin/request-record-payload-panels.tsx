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

type PayloadTabValue =
  | 'requestHeaders'
  | 'requestBody'
  | 'clientResponseHeaders'
  | 'clientResponseBody'
  | 'providerRequestHeaders'
  | 'providerRequestBody'
  | 'providerResponseHeaders'
  | 'providerResponseBody';

type PayloadTab = {
  value: PayloadTabValue;
  label: string;
  emptyText: string;
  payload?: unknown | null;
};

export function RequestRecordPayloadPanels({
  requestHeaders,
  requestBody,
  clientResponseHeaders,
  clientResponseBody,
}: {
  requestHeaders?: unknown | null;
  requestBody?: unknown | null;
  clientResponseHeaders?: unknown | null;
  clientResponseBody?: unknown | null;
}) {
  const { t } = useTranslate('admin');
  return (
    <PayloadTabs
      defaultTab="requestHeaders"
      tabs={[
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
          value: 'clientResponseHeaders',
          label: t('requestRecords.clientResponseHeaders'),
          emptyText: t('requestRecords.noClientResponseHeaders'),
          payload: clientResponseHeaders,
        },
        {
          value: 'clientResponseBody',
          label: t('requestRecords.clientResponseBody'),
          emptyText: t('requestRecords.noClientResponseBody'),
          payload: clientResponseBody,
        },
      ]}
    />
  );
}

export function RequestCandidatePayloadPanels({
  providerRequestHeaders,
  providerRequestBody,
  providerResponseHeaders,
  providerResponseBody,
}: {
  providerRequestHeaders?: unknown | null;
  providerRequestBody?: unknown | null;
  providerResponseHeaders?: unknown | null;
  providerResponseBody?: unknown | null;
}) {
  const { t } = useTranslate('admin');
  return (
    <PayloadTabs
      defaultTab="providerRequestHeaders"
      tabs={[
        {
          value: 'providerRequestHeaders',
          label: t('requestRecords.providerRequestHeaders'),
          emptyText: t('requestRecords.noProviderRequestHeaders'),
          payload: providerRequestHeaders,
        },
        {
          value: 'providerRequestBody',
          label: t('requestRecords.providerRequestBody'),
          emptyText: t('requestRecords.noProviderRequestBody'),
          payload: providerRequestBody,
        },
        {
          value: 'providerResponseHeaders',
          label: t('requestRecords.providerResponseHeaders'),
          emptyText: t('requestRecords.noProviderResponseHeaders'),
          payload: providerResponseHeaders,
        },
        {
          value: 'providerResponseBody',
          label: t('requestRecords.providerResponseBody'),
          emptyText: t('requestRecords.noProviderResponseBody'),
          payload: providerResponseBody,
        },
      ]}
    />
  );
}

function PayloadTabs({
  tabs,
  defaultTab,
}: {
  tabs: PayloadTab[];
  defaultTab: PayloadTabValue;
}) {
  const [activeTab, setActiveTab] = useState<PayloadTabValue>(defaultTab);
  const activePanel = tabs.find((tab) => tab.value === activeTab) ?? tabs[0];

  if (!activePanel) {
    throw new Error('Request record payload tabs cannot be empty');
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
