'use client';

import type { RequestRecord } from 'src/types/provider';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { formatCost } from './request-records-utils';

export function RequestRecordBillingDetails({
  record,
  currencyDisplay,
}: {
  record: RequestRecord | null;
  currencyDisplay: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const costItems = billingCostItems(record, currencyDisplay, t);
  const summaryItems = billingSummaryItems(record, currencyDisplay, t);

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('requestRecords.costDetails')}</Typography>
      <DetailRow items={costItems} />
      <DetailRow items={summaryItems} />
    </Stack>
  );
}

function DetailRow({ items }: { items: string[][] }) {
  return (
    <Stack direction="row" spacing={2} useFlexGap flexWrap="wrap">
      {items.map(([label, value]) => (
        <Stack key={label} spacing={0.25}>
          <Typography variant="caption" color="text.secondary">
            {label}
          </Typography>
          <Typography variant="body2">{value}</Typography>
        </Stack>
      ))}
    </Stack>
  );
}

function billingCostItems(
  record: RequestRecord | null,
  currencyDisplay: CurrencyDisplay,
  t: (key: string) => string
) {
  return [
    [t('requestRecords.inputCost'), formatCost(record?.input_cost, currencyDisplay)],
    [t('requestRecords.outputCost'), formatCost(record?.output_cost, currencyDisplay)],
    [t('requestRecords.inputPrice'), tokenPrice(record?.input_price_per_million, currencyDisplay)],
    [t('requestRecords.outputPrice'), tokenPrice(record?.output_price_per_million, currencyDisplay)],
    [t('requestRecords.cacheReadCost'), formatCost(record?.cache_read_cost, currencyDisplay)],
  ];
}

function billingSummaryItems(
  record: RequestRecord | null,
  currencyDisplay: CurrencyDisplay,
  t: (key: string) => string
) {
  return [
    [t('requestRecords.serviceTier'), serviceTierLabel(record?.service_tier, t)],
    [t('requestRecords.rate'), `${formatMultiplier(record?.billing_multiplier)}x`],
    [t('requestRecords.original'), formatCost(record?.base_cost, currencyDisplay)],
    [t('requestRecords.billed'), formatCost(record?.total_cost, currencyDisplay)],
  ];
}

function tokenPrice(value: number | null | undefined, currencyDisplay: CurrencyDisplay) {
  return `${formatCost(value, currencyDisplay)} /1M Token`;
}

function formatMultiplier(value: number | null | undefined) {
  const normalized = Number(value ?? 1);
  if (!Number.isFinite(normalized)) return '1';
  return normalized.toFixed(4).replace(/\.?0+$/, '');
}

function serviceTierLabel(value: string | null | undefined, t: (key: string) => string) {
  const tier = (value || 'standard').trim().toLowerCase();
  if (tier === 'fast' || tier === 'priority') return t('requestRecords.serviceTierPriority');
  if (tier === 'flex') return t('requestRecords.serviceTierFlex');
  if (tier === 'default' || tier === 'standard') return t('requestRecords.serviceTierStandard');
  return value || t('requestRecords.serviceTierStandard');
}

