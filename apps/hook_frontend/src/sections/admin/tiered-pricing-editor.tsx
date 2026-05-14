'use client';

import type { PricingTier, TieredPricingConfig } from 'src/types/model';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { accountingCurrencyLabel } from 'src/utils/money-boundary';
import {
  CACHE_1H_TTL_MINUTES,
  aetherCacheReadPrice,
  aetherCacheCreationPrice,
  aetherCache1hCreationPrice,
} from 'src/utils/model-pricing';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import {
  addTier,
  updateTier,
  removeTier,
  formatTokens,
  tierStartLabel,
  validatePricingConfig,
} from './tiered-pricing-utils';

// ----------------------------------------------------------------------

type Props = {
  pricing: TieredPricingConfig;
  onChange: (pricing: TieredPricingConfig) => void;
};

export function TieredPricingEditor({ pricing, onChange }: Props) {
  const { t } = useTranslate('admin');
  const validation = useMemo(() => validatePricingConfig(pricing), [pricing]);

  return (
    <Stack spacing={1.5}>
      {pricing.tiers.map((tier, index) => (
        <TierRow
          key={`${tier.up_to ?? 'open'}-${index}`}
          pricing={pricing}
          tier={tier}
          index={index}
          onChange={onChange}
        />
      ))}

      <Button
        fullWidth
        variant="outlined"
        startIcon={<Iconify icon="mingcute:add-line" />}
        onClick={() => onChange(addTier(pricing))}
      >
        {t('actions.addPricingTier')}
      </Button>

      {validation && <Alert severity="error">{t(`models.pricingErrors.${validation}`)}</Alert>}
    </Stack>
  );
}

function TierRow({
  pricing,
  tier,
  index,
  onChange,
}: {
  pricing: TieredPricingConfig;
  tier: PricingTier;
  index: number;
  onChange: (pricing: TieredPricingConfig) => void;
}) {
  return (
    <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: 1.5 }}>
      <TierHeader pricing={pricing} tier={tier} index={index} onChange={onChange} />
      <Divider sx={{ my: 1.5 }} />
      <TierPriceFields pricing={pricing} tier={tier} index={index} onChange={onChange} />
    </Box>
  );
}

function TierHeader({
  pricing,
  tier,
  index,
  onChange,
}: {
  pricing: TieredPricingConfig;
  tier: PricingTier;
  index: number;
  onChange: (pricing: TieredPricingConfig) => void;
}) {
  const { t } = useTranslate('admin');
  const isLast = index === pricing.tiers.length - 1;

  return (
    <Stack direction="row" alignItems="center" spacing={1}>
      <Box sx={{ minWidth: 0, flex: 1 }}>
        <Typography variant="subtitle2">{t('models.tierTitle', { index: index + 1 })}</Typography>
        <Typography variant="caption" color="text.secondary">
          {tierStartLabel(pricing.tiers, index)} - {isLast ? t('models.tierUnlimited') : formatTokens(tier.up_to)}
        </Typography>
      </Box>
      {!isLast && (
        <TextField
          size="small"
          type="number"
          label={t('fields.tierLimit')}
          value={tier.up_to ?? ''}
          onChange={(event) => onChange(updateTier(pricing, index, { up_to: toInteger(event.target.value) }))}
          sx={{ width: 140 }}
        />
      )}
      {pricing.tiers.length > 1 && (
        <IconButton color="error" onClick={() => onChange(removeTier(pricing, index))}>
          <Iconify icon="solar:trash-bin-trash-bold" />
        </IconButton>
      )}
    </Stack>
  );
}

function TierPriceFields({
  pricing,
  tier,
  index,
  onChange,
}: {
  pricing: TieredPricingConfig;
  tier: PricingTier;
  index: number;
  onChange: (pricing: TieredPricingConfig) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr', lg: 'repeat(5, 1fr)' }, gap: 1.5 }}>
      <PriceField
        required
        label={accountingCurrencyLabel(t('fields.inputPrice'))}
        value={tier.input_price_per_1m}
        onChange={(value) => onChange(updateTier(pricing, index, inputPricePatch(tier, value)))}
      />
      <PriceField
        required
        label={accountingCurrencyLabel(t('fields.outputPrice'))}
        value={tier.output_price_per_1m}
        onChange={(value) => onChange(updateTier(pricing, index, { output_price_per_1m: toNumber(value) }))}
      />
      <PriceField
        label={accountingCurrencyLabel(t('fields.cacheCreationPrice'))}
        value={tier.cache_creation_price_per_1m ?? ''}
        onChange={(value) => onChange(updateTier(pricing, index, { cache_creation_price_per_1m: toNumber(value) }))}
      />
      <PriceField
        label={accountingCurrencyLabel(t('fields.cacheReadPrice'))}
        value={tier.cache_read_price_per_1m ?? ''}
        onChange={(value) => onChange(updateTier(pricing, index, { cache_read_price_per_1m: toNumber(value) }))}
      />
      <PriceField
        label={accountingCurrencyLabel(t('models.oneHourCache'))}
        value={oneHourValue(tier)}
        onChange={(value) => onChange(updateTier(pricing, index, oneHourPatchForTier(tier, value)))}
      />
    </Box>
  );
}

function PriceField({
  label,
  value,
  required,
  onChange,
}: {
  label: string;
  value: string | number;
  required?: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      fullWidth
      required={required}
      size="small"
      type="number"
      label={label}
      value={value}
      onChange={(event) => onChange(event.target.value)}
      inputProps={{ min: 0, step: 0.0001 }}
    />
  );
}

function inputPricePatch(tier: PricingTier, value: string): Partial<PricingTier> {
  const oldInput = tier.input_price_per_1m;
  const nextInput = toNumber(value);

  return {
    input_price_per_1m: nextInput,
    cache_creation_price_per_1m: nextCacheValue({
      current: tier.cache_creation_price_per_1m,
      previousAuto: aetherCacheCreationPrice(oldInput),
      nextAuto: aetherCacheCreationPrice(nextInput),
    }),
    cache_read_price_per_1m: nextCacheValue({
      current: tier.cache_read_price_per_1m,
      previousAuto: aetherCacheReadPrice(oldInput),
      nextAuto: aetherCacheReadPrice(nextInput),
    }),
    cache_ttl_pricing: oneHourCachePatchForInput(tier, oldInput, nextInput),
  };
}

function oneHourPatchForTier(tier: PricingTier, value: string): Partial<PricingTier> {
  const others = otherTtlPrices(tier);
  return {
    cache_ttl_pricing: [
      ...others,
      {
        ttl_minutes: CACHE_1H_TTL_MINUTES,
        cache_creation_price_per_1m: toNumber(value),
      },
    ],
  };
}

function oneHourValue(tier: PricingTier) {
  return (
    tier.cache_ttl_pricing?.find((item) => item.ttl_minutes === CACHE_1H_TTL_MINUTES)
      ?.cache_creation_price_per_1m ?? ''
  );
}

function oneHourCacheForInput(tier: PricingTier, oldInput: number, nextInput: number) {
  const current = oneHourValue(tier);
  const next = nextCacheValue({
    current: typeof current === 'number' ? current : undefined,
    previousAuto: aetherCache1hCreationPrice(oldInput),
    nextAuto: aetherCache1hCreationPrice(nextInput),
  });

  return {
    ttl_minutes: CACHE_1H_TTL_MINUTES,
    cache_creation_price_per_1m: next ?? 0,
  };
}

function oneHourCachePatchForInput(tier: PricingTier, oldInput: number, nextInput: number) {
  return [...otherTtlPrices(tier), oneHourCacheForInput(tier, oldInput, nextInput)];
}

function otherTtlPrices(tier: PricingTier) {
  return tier.cache_ttl_pricing?.filter((item) => item.ttl_minutes !== CACHE_1H_TTL_MINUTES) ?? [];
}

function nextCacheValue({
  current,
  previousAuto,
  nextAuto,
}: {
  current?: number | null;
  previousAuto?: number;
  nextAuto?: number;
}) {
  if (current === null || current === undefined) return nextAuto;
  return current === previousAuto ? nextAuto : current;
}

function toNumber(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function toInteger(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.trunc(parsed) : 0;
}
