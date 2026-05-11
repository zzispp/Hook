'use client';

import type { Theme } from '@mui/material/styles';
import type { RequestRecord, RequestCandidateDetail } from 'src/types/provider';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { useRequestRecordDetail } from 'src/actions/request-records';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { formatApiFormat } from './provider-management-utils';
import {
  compactId,
  formatCost,
  tokenDisplay,
  formatDuration,
  formatRequestDate,
  billingStatusLabel,
  requestStatusColor,
  requestStatusLabel,
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

  return (
    <Drawer anchor="right" open={open} onClose={onClose} slotProps={drawerSlotProps}>
      <DrawerHeader
        record={detail.data?.record ?? record}
        locale={locale}
        loading={detail.isLoading}
        onRefresh={detail.refresh}
        onClose={onClose}
      />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, pb: 5 }}>
          <CostSummary record={detail.data?.record ?? record} />
          <TraceSection candidates={detail.data?.candidates ?? []} loading={detail.isLoading} locale={locale} />
          <RequestBody body={detail.data?.request_body} />
        </Stack>
      </Scrollbar>
    </Drawer>
  );
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
  const items = [
    `ID: ${compactId(record.request_id)}`,
    formatRequestDate(record.created_at, locale),
    formatApiFormat(record.client_api_format),
    `${t('requestRecords.user')}: ${record.username || '-'}`,
    tokenDisplay(record),
  ];

  return (
    <Typography variant="caption" color="text.secondary">
      {items.join(' | ')}
    </Typography>
  );
}

function CostSummary({ record }: { record: RequestRecord | null }) {
  const { t } = useTranslate('admin');
  const metrics = [
    [t('requestRecords.totalCost'), formatCost(record?.total_cost)],
    [t('requestRecords.actualCost'), formatCost(0)],
    [t('requestRecords.profit'), formatCost(0)],
    [t('requestRecords.profitRate'), '0.00%'],
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
      <Typography variant="caption" color="text.secondary">
        {t('requestRecords.billingFormula')}
      </Typography>
    </Stack>
  );
}

function TraceSection({
  candidates,
  loading,
  locale,
}: {
  candidates: RequestCandidateDetail[];
  loading: boolean;
  locale: string;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Typography variant="subtitle2">{t('requestRecords.traceTitle')}</Typography>
      {loading ? <Typography variant="body2">{t('common.loading')}</Typography> : null}
      {!loading && candidates.length === 0 ? <Typography variant="body2">{t('common.noData')}</Typography> : null}
      {candidates.map((candidate, index) => (
        <Stack key={candidate.id} spacing={1} sx={traceItemSx}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
            <Typography variant="body2" sx={{ fontWeight: 700 }}>
              {candidate.provider_name || t('requestRecords.unknownProvider')}
            </Typography>
            <Label color={requestStatusColor(candidate.status)} variant="soft">
              {candidate.status_code ?? candidate.status}
            </Label>
          </Stack>
          <Typography variant="caption" color="text.secondary">
            {index + 1} / {candidates.length} | {formatApiFormat(candidate.provider_api_format || candidate.client_api_format)} | {candidate.key_name || '-'} {candidate.key_preview || ''}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {candidate.started_at ? formatRequestDate(candidate.started_at, locale) : '-'} {'->'} {candidate.finished_at ? formatRequestDate(candidate.finished_at, locale) : t('requestRecords.inProgress')}
          </Typography>
          {candidate.error_message ? <Typography variant="caption" color="error">{candidate.error_message}</Typography> : null}
        </Stack>
      ))}
    </Stack>
  );
}

function RequestBody({ body }: { body?: Record<string, unknown> | null }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Typography variant="subtitle2">{t('requestRecords.requestBody')}</Typography>
      <Typography variant="body2" color="text.secondary" sx={{ whiteSpace: 'pre-wrap', fontFamily: body ? 'monospace' : undefined }}>
        {body ? JSON.stringify(body, null, 2) : t('requestRecords.noRequestBody')}
      </Typography>
    </Stack>
  );
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

const traceItemSx = {
  p: 1.5,
  borderRadius: 1,
  bgcolor: 'background.neutral',
};
