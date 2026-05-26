'use client';

import type { Theme } from '@mui/material/styles';
import type { RequestRecord } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { formatCost, formatTokenCount } from './request-records-utils';

type DetailRow = {
  label: string;
  usage: string;
  upstreamPrice: string;
  upstreamCost: string;
  globalPrice: string;
  globalCost: string;
};

type TokenRowInput = {
  label: string;
  usage: number | null | undefined;
  upstreamPrice: number | null | undefined;
  upstreamCost: number | null | undefined;
  globalPrice: number | null | undefined;
  globalCost: number | null | undefined;
};

export function RequestRecordBillingDetails({ record }: { record: RequestRecord | null }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      <CostHeader record={record} />
      <DetailGrid rows={detailRows(record, t)} />
      <BillingFormula record={record} />
    </Stack>
  );
}

function CostHeader({ record }: { record: RequestRecord | null }) {
  const { t } = useTranslate('admin');
  const items = [
    [t('requestRecords.upstreamTotalCost'), formatCost(record?.upstream_total_cost)],
    [t('requestRecords.billed'), formatCost(record?.total_cost)],
    [t('requestRecords.profit'), formatCost(upstreamProfit(record))],
  ];

  return (
    <Stack spacing={1}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
        <Typography variant="subtitle2">{t('requestRecords.costDetails')}</Typography>
        <Chip size="small" variant="soft" label={serviceTierLabel(record?.service_tier, t)} />
      </Stack>
      <Box sx={summaryGridSx}>
        {items.map(([label, value]) => (
          <Stack key={label} spacing={0.25}>
            <Typography variant="caption" color="text.secondary">
              {label}
            </Typography>
            <Typography variant="subtitle2">{value}</Typography>
          </Stack>
        ))}
      </Box>
      <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
        <Chip size="small" label={upstreamSourceLabel(record?.upstream_cost_source, t)} />
        <Chip size="small" label={upstreamModeLabel(record?.upstream_cost_mode, t)} />
        <Chip size="small" label={`${t('requestRecords.rate')} ${formatMultiplier(record?.billing_multiplier)}x`} />
      </Stack>
    </Stack>
  );
}

function DetailGrid({ rows }: { rows: DetailRow[] }) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={tableSx}>
      <GridHeader t={t} />
      {rows.map((row) => (
        <Box key={row.label} sx={rowSx}>
          <Cell label={row.label} value={row.usage} />
          <Cell label={row.upstreamPrice} value={row.upstreamCost} />
          <Cell label={row.globalPrice} value={row.globalCost} />
        </Box>
      ))}
    </Box>
  );
}

function GridHeader({ t }: { t: (key: string) => string }) {
  return (
    <Box sx={headerRowSx}>
      <Typography variant="caption" color="text.secondary">
        {t('requestRecords.usageColumn')}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('requestRecords.upstreamCostSection')}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('requestRecords.globalBillingSection')}
      </Typography>
    </Box>
  );
}

function Cell({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.25} sx={{ minWidth: 0 }}>
      <Typography variant="caption" color="text.secondary" noWrap>
        {label}
      </Typography>
      <Typography variant="body2" sx={{ wordBreak: 'break-word' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function BillingFormula({ record }: { record: RequestRecord | null }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Divider />
      <Stack direction="row" spacing={2} useFlexGap flexWrap="wrap">
        <SmallMetric label={t('requestRecords.original')} value={formatCost(record?.base_cost)} />
        <SmallMetric label={t('requestRecords.rate')} value={`${formatMultiplier(record?.billing_multiplier)}x`} />
        <SmallMetric label={t('requestRecords.billed')} value={formatCost(record?.total_cost)} />
      </Stack>
    </>
  );
}

function SmallMetric({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.25}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2">{value}</Typography>
    </Stack>
  );
}

function detailRows(record: RequestRecord | null, t: (key: string) => string): DetailRow[] {
  if (record?.upstream_cost_mode === 'per_request') return perRequestRows(record, t);
  return tokenRows(record, t);
}

function perRequestRows(record: RequestRecord | null, t: (key: string) => string): DetailRow[] {
  return [
    {
      label: t('requestRecords.pricePerRequest'),
      usage: '1',
      upstreamPrice: formatCost(record?.upstream_price_per_request),
      upstreamCost: formatCost(record?.upstream_request_cost),
      globalPrice: formatCost(record?.request_cost),
      globalCost: formatCost(record?.request_cost),
    },
  ];
}

function tokenRows(record: RequestRecord | null, t: (key: string) => string): DetailRow[] {
  return [
    tokenRow({
      label: t('requestRecords.inputTokens'),
      usage: record?.prompt_tokens,
      upstreamPrice: record?.upstream_input_price_per_million,
      upstreamCost: record?.upstream_input_cost,
      globalPrice: record?.input_price_per_million,
      globalCost: record?.input_cost,
    }),
    tokenRow({
      label: t('requestRecords.outputTokens'),
      usage: record?.completion_tokens,
      upstreamPrice: record?.upstream_output_price_per_million,
      upstreamCost: record?.upstream_output_cost,
      globalPrice: record?.output_price_per_million,
      globalCost: record?.output_cost,
    }),
    tokenRow({
      label: t('requestRecords.cacheCreationTokens'),
      usage: record?.cache_creation_input_tokens,
      upstreamPrice: record?.upstream_cache_creation_price_per_million,
      upstreamCost: record?.upstream_cache_creation_cost,
      globalPrice: record?.cache_creation_price_per_million,
      globalCost: record?.cache_creation_cost,
    }),
    tokenRow({
      label: t('requestRecords.cacheReadTokens'),
      usage: record?.cache_read_input_tokens,
      upstreamPrice: record?.upstream_cache_read_price_per_million,
      upstreamCost: record?.upstream_cache_read_cost,
      globalPrice: record?.cache_read_price_per_million,
      globalCost: record?.cache_read_cost,
    }),
  ];
}

function tokenRow(input: TokenRowInput): DetailRow {
  return {
    label: input.label,
    usage: cacheToken(input.usage),
    upstreamPrice: tokenPrice(input.upstreamPrice),
    upstreamCost: formatCost(input.upstreamCost),
    globalPrice: tokenPrice(input.globalPrice),
    globalCost: formatCost(input.globalCost),
  };
}

function upstreamProfit(record: RequestRecord | null) {
  return Number(record?.total_cost ?? 0) - Number(record?.upstream_total_cost ?? 0);
}

function tokenPrice(value: number | null | undefined) {
  if (value === null || value === undefined) return '-';
  return `${formatCost(value)} /1M`;
}

function cacheToken(value: number | null | undefined) {
  return value ? formatTokenCount(value) : '-';
}

function formatMultiplier(value: number | null | undefined) {
  const normalized = Number(value ?? 1);
  if (!Number.isFinite(normalized)) return '1';
  return normalized.toFixed(4).replace(/\.?0+$/, '');
}

function upstreamSourceLabel(value: string | null | undefined, t: (key: string) => string) {
  if (value === 'configured') return t('requestRecords.upstreamConfigured');
  if (value === 'global_default') return t('requestRecords.upstreamGlobalDefault');
  return '-';
}

function upstreamModeLabel(value: string | null | undefined, t: (key: string) => string) {
  if (value === 'per_request') return t('requestRecords.perRequestMode');
  if (value === 'per_token') return t('requestRecords.perTokenMode');
  return '-';
}

function serviceTierLabel(value: string | null | undefined, t: (key: string) => string) {
  const tier = (value || 'standard').trim().toLowerCase();
  if (tier === 'fast' || tier === 'priority') return t('requestRecords.serviceTierPriority');
  if (tier === 'flex') return t('requestRecords.serviceTierFlex');
  if (tier === 'default' || tier === 'standard') return t('requestRecords.serviceTierStandard');
  return value || t('requestRecords.serviceTierStandard');
}

const summaryGridSx = {
  display: 'grid',
  gap: 1.5,
  gridTemplateColumns: { xs: '1fr', sm: 'repeat(3, minmax(0, 1fr))' },
};

const tableSx = {
  overflow: 'hidden',
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

const headerRowSx = {
  px: 1.25,
  py: 0.75,
  display: 'grid',
  gap: 1,
  gridTemplateColumns: '1fr 1fr 1fr',
  bgcolor: 'background.neutral',
};

const rowSx = {
  px: 1.25,
  py: 1,
  display: 'grid',
  gap: 1,
  gridTemplateColumns: '1fr 1fr 1fr',
  borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};
