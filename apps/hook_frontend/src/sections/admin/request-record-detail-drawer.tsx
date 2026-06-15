'use client';

import type { Theme } from '@mui/material/styles';
import type { RequestRecord } from 'src/types/provider';

import { useMemo, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { useRoutingDecision } from 'src/actions/routing';
import { useRequestRecordDetail } from 'src/actions/request-records';

import { Label } from 'src/components/label';
import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { RequestRecordTraceTimeline } from './request-record-trace-timeline';
import { RequestRecordPayloadPanels } from './request-record-payload-panels';
import { RequestRecordBillingDetails } from './request-record-billing-details';
import {
  formatCost,
  userDisplay,
  formatDuration,
  formatTokenCount,
  formatRequestDate,
  billingStatusLabel,
  requestStatusColor,
  requestStatusLabel,
  formatRequestApiFormat,
} from './request-records-utils';

export function RequestRecordDetailDrawer({
  open,
  record,
  locale,
  onClose,
}: {
  open: boolean;
  record: RequestRecord | null;
  locale: string;
  onClose: VoidFunction;
}) {
  const detail = useRequestRecordDetail(open ? record?.request_id : null);
  const routingDecision = useRoutingDecision(open ? record?.request_id : null);
  const displayRecord = useMemo(
    () => freshestRecord(record, detail.data?.record),
    [detail.data?.record, record]
  );

  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      disableScrollLock
      disableRestoreFocus
      slotProps={drawerSlotProps}
    >
      <DrawerHeader
        record={displayRecord}
        locale={locale}
        loading={detail.isLoading}
        onRefresh={detail.refresh}
        onClose={onClose}
      />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, pb: 5 }}>
          <CostSummary record={displayRecord} />
          <RequestRecordTraceTimeline
            record={displayRecord}
            candidates={detail.data?.candidates ?? []}
            routingDecision={routingDecision.data ?? null}
            routingDecisionError={routingDecision.error}
            routingDecisionLoading={routingDecision.isLoading}
            loading={detail.isLoading}
            locale={locale}
          />
          <RequestRecordPayloadPanels
            requestHeaders={detail.data?.request_headers}
            requestBody={detail.data?.request_body}
            clientResponseHeaders={detail.data?.client_response_headers}
            clientResponseBody={detail.data?.client_response_body}
          />
        </Stack>
      </Scrollbar>
    </Drawer>
  );
}

const RECORD_STATUS_RANK: Record<string, number> = {
  pending: 0,
  streaming: 1,
  success: 2,
  failed: 2,
  cancelled: 2,
};

function freshestRecord(base: RequestRecord | null, detail?: RequestRecord) {
  if (!base) return detail ?? null;
  if (!detail) return base;
  if (recordStatusRank(detail.status) > recordStatusRank(base.status)) return detail;
  return base;
}

function recordStatusRank(status: string) {
  return RECORD_STATUS_RANK[status] ?? 0;
}

function DrawerHeader({
  record,
  locale,
  loading,
  onClose,
  onRefresh,
}: {
  record: RequestRecord | null;
  locale: string;
  loading: boolean;
  onClose: VoidFunction;
  onRefresh: VoidFunction;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={headerSx}>
      <Stack spacing={1} sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" spacing={1} alignItems="center" useFlexGap flexWrap="wrap">
          <Typography variant="h6">{t('requestRecords.detailTitle')}</Typography>
          <Typography variant="subtitle2" sx={{ fontFamily: 'monospace' }}>
            {record?.model_name || '-'}
          </Typography>
          {record ? <RequestStatusLabel record={record} /> : null}
        </Stack>
        {record ? <HeaderMeta record={record} locale={locale} /> : null}
      </Stack>
      <Tooltip title={t('common.refresh')}>
        <IconButton disabled={loading} onClick={onRefresh}>
          <Iconify icon="solar:restart-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function HeaderMeta({ record, locale }: { record: RequestRecord; locale: string }) {
  const { t } = useTranslate('admin');
  const { copy } = useCopyToClipboard();
  const handleCopy = useCallback(() => {
    copy(record.request_id);
    toast.success(t('requestRecords.requestIdCopied'));
  }, [copy, record.request_id, t]);
  const items = [
    formatRequestDate(record.created_at, locale),
    formatRequestApiFormat(record),
    `${t('requestRecords.user')}: ${userDisplay(record)}`,
    `${formatTokenCount(record.prompt_tokens)} / ${formatTokenCount(record.completion_tokens)}`,
  ];

  return (
    <Stack direction="row" alignItems="center" spacing={0.75} useFlexGap flexWrap="wrap">
      <Stack direction="row" alignItems="center" spacing={0.25} sx={{ minWidth: 0 }}>
        <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace', overflowWrap: 'anywhere' }}>
          ID: {record.request_id}
        </Typography>
        <Tooltip title={t('requestRecords.copyRequestId')}>
          <IconButton size="small" onClick={handleCopy} sx={{ width: 24, height: 24 }}>
            <Iconify icon="solar:copy-bold" width={14} />
          </IconButton>
        </Tooltip>
      </Stack>
      <Typography variant="caption" color="text.secondary">
        {items.join(' | ')}
      </Typography>
    </Stack>
  );
}

function CostSummary({ record }: { record: RequestRecord | null }) {
  const { t } = useTranslate('admin');
  const metrics = [
    [t('requestRecords.totalCost'), formatCost(record?.total_cost)],
    [t('requestRecords.actualCost'), formatCost(record?.upstream_total_cost)],
    [t('requestRecords.profit'), formatCost(profit(record))],
    [t('requestRecords.profitRate'), profitRate(record)],
    [t('requestRecords.responseTime'), formatDuration(record?.total_latency_ms)],
  ];

  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Stack direction="row" spacing={2} useFlexGap flexWrap="wrap">
        {metrics.map(([label, value]) => (
          <Stack key={label} spacing={0.25}>
            <Typography variant="caption" color="text.secondary">
              {label}
            </Typography>
            <Typography variant="subtitle2">{value}</Typography>
          </Stack>
        ))}
      </Stack>
      <Divider />
      <RequestRecordBillingDetails record={record} />
    </Stack>
  );
}

function profit(record: RequestRecord | null) {
  return Number(record?.total_cost ?? 0) - Number(record?.upstream_total_cost ?? 0);
}

function profitRate(record: RequestRecord | null) {
  const total = Number(record?.total_cost ?? 0);
  if (total <= 0) return '0.00%';
  return `${((profit(record) / total) * 100).toFixed(2)}%`;
}

function RequestStatusLabel({ record }: { record: RequestRecord }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Label color={requestStatusColor(record.status)} variant="soft">
        {requestStatusLabel(record.status, t)}
      </Label>
      <Label color="default" variant="soft">
        {record.is_stream ? t('requestRecords.stream') : t('requestRecords.nonStream')}
      </Label>
      <Label color="default" variant="soft">
        {billingStatusLabel(record.billing_status, t)}
      </Label>
    </>
  );
}

const drawerSlotProps = {
  backdrop: { invisible: true },
  paper: {
    sx: [
      (theme: Theme) => ({
        ...theme.mixins.paperStyles(theme, {
          color: varAlpha(theme.vars.palette.background.defaultChannel, 0.95),
        }),
        width: { xs: 1, sm: 760 },
      }),
    ],
  },
};

const headerSx = {
  py: 2,
  pr: 1,
  pl: 2.5,
  gap: 1,
  display: 'flex',
  alignItems: 'flex-start',
};

const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};
