'use client';

import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { PricingTier, GlobalModelResponse } from 'src/types/model';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { formatPrice } from './model-catalog-utils';

type Props = {
  model: GlobalModelResponse;
  multiplier: number;
  currencyDisplay?: CurrencyDisplay;
  title?: string;
};

export function ModelEffectivePricingSection({
  model,
  multiplier,
  currencyDisplay,
  title,
}: Props) {
  return (
    <Stack spacing={1}>
      {title ? <SectionTitle title={title} /> : null}
      {model.default_tiered_pricing.tiers.map((tier, index) => (
        <TierPrice
          key={`${tier.up_to ?? 'open'}-${index}`}
          tier={tier}
          index={index}
          multiplier={multiplier}
          currencyDisplay={currencyDisplay}
        />
      ))}
      <RequestPrice
        value={model.default_price_per_request}
        multiplier={multiplier}
        currencyDisplay={currencyDisplay}
      />
    </Stack>
  );
}

function SectionTitle({ title }: { title: string }) {
  return <Typography variant="subtitle2">{title}</Typography>;
}

function TierPrice({
  tier,
  index,
  multiplier,
  currencyDisplay,
}: {
  tier: PricingTier;
  index: number;
  multiplier: number;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const rows = tierRows(tier, multiplier, currencyDisplay, t);

  return (
    <Stack spacing={0.75} sx={innerPanelSx}>
      <Typography variant="caption" color="text.secondary">
        {t('models.tierTitle', { index: index + 1 })}
      </Typography>
      {rows.map(([label, value]) => (
        <PriceLine key={label} label={label} value={value} />
      ))}
    </Stack>
  );
}

function RequestPrice({
  value,
  multiplier,
  currencyDisplay,
}: {
  value?: number | null;
  multiplier: number;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  if (!value || value <= 0) return null;

  return (
    <Stack sx={innerPanelSx}>
      <PriceLine
        label={t('fields.pricePerRequest')}
        value={`${multipliedPrice(value, multiplier, currencyDisplay)}/${t('providers.perRequest')}`}
      />
    </Stack>
  );
}

function PriceLine({ label, value }: { label: string; value: string }) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1.5}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ fontFamily: 'monospace', textAlign: 'right' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function tierRows(
  tier: PricingTier,
  multiplier: number,
  currencyDisplay: CurrencyDisplay | undefined,
  t: (key: string) => string
) {
  return [
    [t('fields.inputPrice'), multipliedPrice(tier.input_price_per_1m, multiplier, currencyDisplay)],
    [t('fields.outputPrice'), multipliedPrice(tier.output_price_per_1m, multiplier, currencyDisplay)],
    [t('fields.cacheCreationPrice'), multipliedPrice(tier.cache_creation_price_per_1m, multiplier, currencyDisplay)],
    [t('fields.cacheReadPrice'), multipliedPrice(tier.cache_read_price_per_1m, multiplier, currencyDisplay)],
    [t('models.oneHourCache'), multipliedPrice(oneHourCacheRaw(tier), multiplier, currencyDisplay)],
  ];
}

function multipliedPrice(
  value: number | null | undefined,
  multiplier: number,
  currencyDisplay?: CurrencyDisplay
) {
  return formatPrice(value === null || value === undefined ? value : value * multiplier, currencyDisplay);
}

function oneHourCacheRaw(tier: PricingTier) {
  const price = tier.cache_ttl_pricing?.find((item) => item.ttl_minutes === 60);
  return price?.cache_creation_price_per_1m;
}

const innerPanelSx = {
  p: 1,
  borderRadius: 1,
  bgcolor: 'background.neutral',
};
