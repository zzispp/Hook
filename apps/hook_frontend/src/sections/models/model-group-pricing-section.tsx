'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { PricingTier, GlobalModelResponse } from 'src/types/model';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { formatPrice, requestPrice } from './model-catalog-utils';

type Props = {
  model: GlobalModelResponse;
  groups: BillingGroup[];
  loading: boolean;
  errorMessage?: string;
};

export function ModelGroupPricingSection({ model, groups, loading, errorMessage }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('models.groupPricing')}</Typography>
      <GroupPricingState loading={loading} errorMessage={errorMessage} groups={groups} />
      {!loading && !errorMessage ? (
        <Stack spacing={1.5}>
          {groups.map((group) => (
            <GroupPriceCard key={group.id} model={model} group={group} />
          ))}
        </Stack>
      ) : null}
    </Stack>
  );
}

function GroupPricingState({
  loading,
  errorMessage,
  groups,
}: {
  loading: boolean;
  errorMessage?: string;
  groups: BillingGroup[];
}) {
  const { t } = useTranslate('admin');

  if (loading) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('common.loading')}
      </Typography>
    );
  }
  if (errorMessage) return <Alert severity="error">{errorMessage}</Alert>;
  if (groups.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('models.groupPricingEmpty')}
      </Typography>
    );
  }
  return null;
}

function GroupPriceCard({ model, group }: { model: GlobalModelResponse; group: BillingGroup }) {
  const pricing = model.default_tiered_pricing;

  return (
    <Stack spacing={1.25} sx={panelSx}>
      <GroupHeading group={group} />
      <Stack spacing={1}>
        {pricing.tiers.map((tier, index) => (
          <GroupTierPrice key={`${group.code}-${tier.up_to ?? 'open'}-${index}`} tier={tier} index={index} multiplier={group.billing_multiplier} />
        ))}
        <GroupRequestPrice value={model.default_price_per_request} multiplier={group.billing_multiplier} />
      </Stack>
    </Stack>
  );
}

function GroupHeading({ group }: { group: BillingGroup }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75}>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
        <Typography variant="subtitle2" sx={{ flexGrow: 1, minWidth: 0 }} noWrap>
          {translatedGroupName(group, t)}
        </Typography>
        {group.is_system ? <Label variant="soft">{t('common.system')}</Label> : null}
        <Label color="info" variant="soft">
          x{formatMultiplier(group.billing_multiplier)}
        </Label>
      </Stack>
      <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-word' }}>
        {translatedGroupDescription(group, t)}
      </Typography>
    </Stack>
  );
}

function GroupTierPrice({
  tier,
  index,
  multiplier,
}: {
  tier: PricingTier;
  index: number;
  multiplier: number;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1} sx={innerPanelSx}>
      <Typography variant="caption" color="text.secondary">
        {t('models.tierTitle', { index: index + 1 })}
      </Typography>
      <Grid container spacing={1}>
        <Grid size={{ xs: 12 }}>
          <PriceLine label={t('fields.inputPrice')} value={multipliedPrice(tier.input_price_per_1m, multiplier)} />
        </Grid>
        <Grid size={{ xs: 12 }}>
          <PriceLine label={t('fields.outputPrice')} value={multipliedPrice(tier.output_price_per_1m, multiplier)} />
        </Grid>
        <Grid size={{ xs: 12 }}>
          <PriceLine label={t('fields.cacheCreationPrice')} value={multipliedPrice(tier.cache_creation_price_per_1m, multiplier)} />
        </Grid>
        <Grid size={{ xs: 12 }}>
          <PriceLine label={t('fields.cacheReadPrice')} value={multipliedPrice(tier.cache_read_price_per_1m, multiplier)} />
        </Grid>
      </Grid>
    </Stack>
  );
}

function GroupRequestPrice({ value, multiplier }: { value?: number | null; multiplier: number }) {
  const { t } = useTranslate('admin');
  const price = value ? requestPrice(value * multiplier) : null;
  if (!price) return null;

  return (
    <Box sx={innerPanelSx}>
      <PriceLine label={t('fields.pricePerRequest')} value={price} />
    </Box>
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

function multipliedPrice(value: number | null | undefined, multiplier: number) {
  return formatPrice(value === null || value === undefined ? value : value * multiplier);
}

function formatMultiplier(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}

function translatedGroupName(group: BillingGroup, t: (key: string) => string) {
  return group.is_system && group.code === 'default' ? t('billingGroups.systemName') : group.name;
}

function translatedGroupDescription(group: BillingGroup, t: (key: string) => string) {
  if (group.is_system && group.code === 'default') {
    return t('billingGroups.systemDescription');
  }
  return group.description || group.code;
}

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const innerPanelSx = {
  p: 1,
  borderRadius: 1,
  bgcolor: 'background.neutral',
};
