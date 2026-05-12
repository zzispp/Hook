'use client';

import type { Theme } from '@mui/material/styles';
import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { PricingTier, GlobalModelResponse } from 'src/types/model';

import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import {
  tierCount,
  formatPrice,
  requestPrice,
  firstTierPrice,
  oneHourCachePrice,
  firstOneHourCachePrice,
} from './model-catalog-utils';

export function ModelPricingSection({
  model,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const pricing = model.default_tiered_pricing;

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('models.pricingInfo')}</Typography>
      {tierCount(pricing) <= 1 ? (
        <SingleTierPricing model={model} currencyDisplay={currencyDisplay} />
      ) : (
        <TieredPricing tiers={pricing.tiers} currencyDisplay={currencyDisplay} />
      )}
      <RequestPrice value={model.default_price_per_request} currencyDisplay={currencyDisplay} />
    </Stack>
  );
}

function SingleTierPricing({
  model,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const pricing = model.default_tiered_pricing;
  const items = [
    [t('fields.inputPrice'), firstTierPrice(pricing, 'input_price_per_1m', currencyDisplay)],
    [t('fields.outputPrice'), firstTierPrice(pricing, 'output_price_per_1m', currencyDisplay)],
    [
      t('fields.cacheCreationPrice'),
      firstTierPrice(pricing, 'cache_creation_price_per_1m', currencyDisplay),
    ],
    [
      t('fields.cacheReadPrice'),
      firstTierPrice(pricing, 'cache_read_price_per_1m', currencyDisplay),
    ],
  ];

  return (
    <Grid container spacing={1.5}>
      {items.map(([label, value]) => (
        <Grid key={label} size={{ xs: 12 }}>
          <PriceBox label={label} value={value} />
        </Grid>
      ))}
      <Grid size={{ xs: 12 }}>
        <InlinePrice
          label={t('models.oneHourCache')}
          value={firstOneHourCachePrice(pricing, currencyDisplay)}
        />
      </Grid>
    </Grid>
  );
}

function TieredPricing({
  tiers,
  currencyDisplay,
}: {
  tiers: PricingTier[];
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      {tiers.map((tier, index) => (
        <Stack key={`${tier.up_to ?? 'open'}-${index}`} spacing={1} sx={panelSx}>
          <Typography variant="subtitle2">{tierLabel({ t, tiers, tier, index })}</Typography>
          <Grid container spacing={1}>
            <Grid size={{ xs: 12 }}>
              <PriceBox
                label={t('fields.inputPrice')}
                value={formatPrice(tier.input_price_per_1m, currencyDisplay)}
              />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox
                label={t('fields.outputPrice')}
                value={formatPrice(tier.output_price_per_1m, currencyDisplay)}
              />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox
                label={t('fields.cacheCreationPrice')}
                value={formatPrice(tier.cache_creation_price_per_1m, currencyDisplay)}
              />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox
                label={t('fields.cacheReadPrice')}
                value={formatPrice(tier.cache_read_price_per_1m, currencyDisplay)}
              />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <InlinePrice
                label={t('models.oneHourCache')}
                value={oneHourCachePrice(tier, currencyDisplay)}
              />
            </Grid>
          </Grid>
        </Stack>
      ))}
    </Stack>
  );
}

function RequestPrice({
  value,
  currencyDisplay,
}: {
  value?: number | null;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const price = requestPrice(value, currencyDisplay);
  if (!price) return null;
  return <InlinePrice label={t('fields.pricePerRequest')} value={price} />;
}

function PriceBox({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.75} sx={panelSx}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="subtitle1" sx={{ fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function InlinePrice({ label, value }: { label: string; value: string }) {
  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      spacing={2}
      sx={panelSx}
    >
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function tierLabel({
  t,
  tiers,
  tier,
  index,
}: {
  t: (key: string) => string;
  tiers: PricingTier[];
  tier: PricingTier;
  index: number;
}) {
  if (tier.up_to === null)
    return index === 0 ? t('models.tierAll') : `> ${formatTierLimit(tiers[index - 1]?.up_to)}`;
  const start = index === 0 ? '0' : formatTierLimit(tiers[index - 1]?.up_to);
  return `${start} - ${formatTierLimit(tier.up_to)}`;
}

function formatTierLimit(limit?: number | null) {
  if (!limit) return '0';
  if (limit >= 1000000) return `${(limit / 1000000).toFixed(1)}M`;
  if (limit >= 1000) return `${(limit / 1000).toFixed(0)}K`;
  return String(limit);
}

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};
