'use client';

import type { ProviderModelBinding } from 'src/types/provider';
import type { GlobalModelResponse, TieredPricingConfig } from 'src/types/model';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { formatMoneyCompact } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';
import { updateProviderModel } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type Props = {
  binding: ProviderModelBinding;
  providerId: string;
  model?: GlobalModelResponse;
  onEdit: (model: GlobalModelResponse) => void;
  onTest: (binding: ProviderModelBinding) => void;
};

export function ProviderModelRow({ binding, providerId, model, onEdit, onTest }: Props) {
  const { t } = useTranslate('admin');
  const [toggling, setToggling] = useState(false);
  const active = binding.is_active && model?.is_active !== false;

  const toggleActive = async () => {
    setToggling(true);
    try {
      await updateProviderModel(providerId, binding.id, { is_active: !binding.is_active });
      toast.success(
        !binding.is_active
          ? t('messages.providerModelEnabled')
          : t('messages.providerModelDisabled')
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setToggling(false);
    }
  };

  return (
    <TableRow hover sx={{ '&:last-child td': { borderBottom: 0 } }}>
      <TableCell sx={modelCellSx}>
        <ModelNameCell binding={binding} model={model} active={active} />
      </TableCell>
      <TableCell sx={pricingCellSx}>{pricingLines(model, t)}</TableCell>
      <TableCell sx={actionCellSx}>
        <Stack direction="row" justifyContent="flex-end" spacing={0.5}>
          <IconButton size="small" title={t('providers.testModel')} onClick={() => onTest(binding)}>
            <Iconify icon="solar:play-circle-bold" width={16} />
          </IconButton>
          <IconButton size="small" disabled={!model} title={t('common.edit')} onClick={() => model && onEdit(model)}>
            <Iconify icon="solar:pen-bold" width={16} />
          </IconButton>
          <IconButton
            size="small"
            disabled={toggling}
            title={binding.is_active ? t('providers.disableModel') : t('providers.enableModel')}
            onClick={toggleActive}
          >
            <Iconify icon="ic:round-power-settings-new" width={16} />
          </IconButton>
        </Stack>
      </TableCell>
    </TableRow>
  );
}

function ModelNameCell({
  binding,
  model,
  active,
}: {
  binding: ProviderModelBinding;
  model?: GlobalModelResponse;
  active: boolean;
}) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" spacing={1.25} alignItems="flex-start">
      <Box title={modelStatusTitle(binding, model, t)} sx={statusDotSx(active, binding.is_active)} />
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="subtitle2" noWrap>
          {model?.display_name || model?.name || binding.global_model_id}
        </Typography>
        <Stack direction="row" spacing={0.5} alignItems="center" sx={{ mt: 0.5, minWidth: 0 }}>
          <Typography variant="caption" noWrap sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
            {model?.name || binding.global_model_id}
          </Typography>
          <IconButton size="small" title={t('models.copyModelId')} onClick={() => void copyModelId(model?.name || binding.global_model_id, t)}>
            <Iconify icon="solar:copy-bold" width={14} />
          </IconButton>
        </Stack>
      </Box>
    </Stack>
  );
}

function pricingLines(model: GlobalModelResponse | undefined, t: (key: string) => string) {
  const requestPrice = model?.default_price_per_request;
  const tiers = model?.default_tiered_pricing;
  if (requestPrice && requestPrice > 0) {
    return <PriceGrid rows={[[t('providers.pricePerRequest'), `${formatPrice(requestPrice)}/${t('providers.perRequest')}`]]} />;
  }
  const tier = tiers?.tiers?.[0];
  if (!tier) return <Typography variant="caption">-</Typography>;
  return <PriceGrid rows={tierRows(tier, t)} />;
}

function PriceGrid({ rows }: { rows: string[][] }) {
  return (
    <Box sx={priceGridSx}>
      {rows.map(([label, value]) => (
        <Box key={label} component="span" sx={{ display: 'contents' }}>
          <Typography variant="caption" sx={priceLabelSx}>
            {label}
          </Typography>
          <Typography variant="caption" sx={priceValueSx}>
            {value}
          </Typography>
        </Box>
      ))}
    </Box>
  );
}

function tierRows(tier: TieredPricingConfig['tiers'][number], t: (key: string) => string) {
  const rows = [
    [
      t('providers.inputOutputPrice'),
      `${formatPrice(tier.input_price_per_1m)}/${formatPrice(tier.output_price_per_1m)}`,
    ],
  ];
  if ((tier.cache_creation_price_per_1m ?? 0) > 0 || (tier.cache_read_price_per_1m ?? 0) > 0) {
    rows.push([
      t('providers.cachePrice'),
      `${formatPrice(tier.cache_creation_price_per_1m)}/${formatPrice(tier.cache_read_price_per_1m)}`,
    ]);
  }
  const ttl = tier.cache_ttl_pricing?.find((item) => item.ttl_minutes === 60);
  if ((ttl?.cache_creation_price_per_1m ?? 0) > 0) {
    rows.push([
      t('providers.cache1hCreationPrice'),
      formatPrice(ttl?.cache_creation_price_per_1m),
    ]);
  }
  return rows;
}

function modelStatusTitle(
  binding: ProviderModelBinding,
  model: GlobalModelResponse | undefined,
  t: (key: string) => string
) {
  if (!binding.is_active) return t('common.disabled');
  if (model?.is_active === false) return t('providers.globalModelDisabled');
  return t('providers.activeAndAvailable');
}

async function copyModelId(modelId: string, t: (key: string) => string) {
  try {
    await navigator.clipboard.writeText(modelId);
    toast.success(t('models.modelIdCopied'));
  } catch {
    toast.error(t('messages.copyFailed'));
  }
}

function formatPrice(value: number | null | undefined) {
  if (value === null || value === undefined) return '-';
  return formatMoneyCompact(value);
}

const modelCellSx = { verticalAlign: 'top', px: 2, py: 1.5 };
const pricingCellSx = { verticalAlign: 'top', px: 2, py: 1.5, whiteSpace: 'nowrap' };
const actionCellSx = { verticalAlign: 'top', px: 2, py: 1.5 };
const priceGridSx = {
  display: 'grid',
  gridTemplateColumns: 'max-content max-content',
  columnGap: 1,
  rowGap: 0.5,
};
const priceLabelSx = { textAlign: 'left', color: 'text.secondary', minWidth: 0 };
const priceValueSx = { fontFamily: 'monospace', fontWeight: 600 };

function statusDotSx(active: boolean, bindingActive: boolean) {
  return {
    width: 8,
    height: 8,
    mt: 0.75,
    borderRadius: '50%',
    flexShrink: 0,
    bgcolor: active ? 'success.main' : bindingActive ? 'error.main' : 'text.disabled',
  };
}
