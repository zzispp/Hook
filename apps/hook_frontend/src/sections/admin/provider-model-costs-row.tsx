'use client';

import type { Theme } from '@mui/material/styles';
import type { ModelCostRowItem } from './provider-model-cost-types';
import type { CostRatioInfo, CostRatioDetail } from './provider-model-cost-ratio';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { formatMoneyCompact } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';
import { deleteProviderModelCost } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { costRatioInfo } from './provider-model-cost-ratio';

export function ProviderModelCostsRow({
  providerId,
  row,
}: {
  providerId: string;
  row: ModelCostRowItem;
}) {
  const { t } = useTranslate('admin');
  const [deleting, setDeleting] = useState(false);
  const ratio = costRatioInfo({ binding: row.binding, cost: row.cost, models: row.models, t });

  const deleteCost = async () => {
    if (!row.cost || deleting) return;
    setDeleting(true);
    try {
      await deleteProviderModelCost(providerId, row.key.id, row.binding.id);
      toast.success(t('messages.providerModelCostDeleted'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Box sx={rowSx}>
      <Stack direction="row" justifyContent="space-between" spacing={1.5} alignItems="flex-start">
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap>{row.modelLabel}</Typography>
          <Typography variant="caption" sx={monoSx}>{row.binding.provider_model_name}</Typography>
        </Box>
        <Stack direction="row" alignItems="center" spacing={0.75}>
          <Chip size="small" color={row.cost ? 'success' : 'default'} label={sourceLabel(row.source, t)} />
          <RatioChip ratio={ratio} />
          {row.cost ? (
            <Tooltip title={t('common.delete')}>
              <IconButton size="small" disabled={deleting} onClick={deleteCost}>
                <Iconify icon="solar:trash-bin-trash-bold" width={16} />
              </IconButton>
            </Tooltip>
          ) : null}
        </Stack>
      </Stack>
      <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap" sx={metaSx}>
        <Chip size="small" variant="outlined" label={modeLabel(row.mode, t)} />
        <Typography variant="caption" sx={priceSx}>{priceSummary(row, t)}</Typography>
      </Stack>
    </Box>
  );
}

function RatioChip({ ratio }: { ratio: CostRatioInfo }) {
  const { t } = useTranslate('admin');
  const heading = ratio.reasonKey === 'conflict'
    ? t('providers.multiplierConflictReason')
    : ratio.reasonKey === 'unavailable'
      ? t('providers.multiplierUnavailableReason')
      : t('providers.costMultiplier');

  return (
    <Tooltip title={<RatioTooltipContent heading={heading} ratio={ratio} />}>
      <Chip
        size="small"
        variant="outlined"
        color={ratio.reasonKey === 'uniform' ? 'primary' : 'warning'}
        label={ratio.label}
      />
    </Tooltip>
  );
}

function RatioTooltipContent({
  heading,
  ratio,
}: {
  heading: string;
  ratio: CostRatioInfo;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75} sx={{ py: 0.5 }}>
      <Typography variant="subtitle2">{heading}</Typography>
      {ratio.details.map((detail) => (
        <Stack key={detail.fieldKey} spacing={0.25}>
          <Typography variant="caption" sx={{ fontWeight: 700 }}>
            {t(detail.labelKey)}
          </Typography>
          <Typography variant="caption">
            {t('providers.multiplierDetail', {
              globalPrice: displayPrice(detail.globalPrice),
              configuredPrice: displayPrice(detail.configuredPrice),
              ratio: ratioValue(detail, t),
            })}
          </Typography>
        </Stack>
      ))}
    </Stack>
  );
}

function ratioValue(detail: CostRatioDetail, t: (key: string, options?: Record<string, unknown>) => string) {
  if (detail.formattedRatio) return `${detail.formattedRatio}x`;
  return reasonLabel(detail.unavailableReasonKey, t);
}

function reasonLabel(
  reasonKey: CostRatioDetail['unavailableReasonKey'],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (reasonKey === 'missing_global') return t('providers.multiplierUnavailableReasons.missing_global');
  if (reasonKey === 'missing_configured') return t('providers.multiplierUnavailableReasons.missing_configured');
  if (reasonKey === 'non_positive_global') return t('providers.multiplierUnavailableReasons.non_positive_global');
  if (reasonKey === 'non_positive_configured') return t('providers.multiplierUnavailableReasons.non_positive_configured');
  return t('providers.multiplierUnavailableReason');
}

function priceSummary(row: ModelCostRowItem, t: (key: string) => string) {
  if (row.mode === 'per_request') {
    return `${t('providers.pricePerRequest')}: ${displayPrice(row.requestPrice)}`;
  }
  return [
    `${t('requestRecords.inputPrice')}: ${displayPrice(numberOrNull(row.tokenDraft.input_price_per_million))}`,
    `${t('requestRecords.outputPrice')}: ${displayPrice(numberOrNull(row.tokenDraft.output_price_per_million))}`,
    `${t('requestRecords.cacheCreationPrice')}: ${displayPrice(numberOrNull(row.tokenDraft.cache_creation_price_per_million))}`,
    `${t('requestRecords.cacheReadPrice')}: ${displayPrice(numberOrNull(row.tokenDraft.cache_read_price_per_million))}`,
  ].join(' / ');
}

function modeLabel(mode: string, t: (key: string) => string) {
  return mode === 'per_request' ? t('providers.perRequestCost') : t('providers.perTokenCost');
}

function sourceLabel(source: string, t: (key: string) => string) {
  return source === 'configured' ? t('providers.configuredCost') : t('providers.globalDefaultCost');
}

function displayPrice(value: number | null | undefined) {
  return value === null || value === undefined ? '-' : formatMoneyCompact(value);
}

function numberOrNull(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

const rowSx = { px: 2, py: 1.5, transition: (theme: Theme) => theme.transitions.create('background-color'), '&:hover': { bgcolor: 'action.hover' } };
const monoSx = { fontFamily: 'monospace', color: 'text.secondary' };
const metaSx = { mt: 1, alignItems: 'center', color: 'text.secondary' };
const priceSx = { fontFamily: 'monospace', color: 'text.secondary' };
