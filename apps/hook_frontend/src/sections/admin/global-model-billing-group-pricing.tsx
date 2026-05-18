'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { ModelEffectivePricingSection } from '../models/model-effective-pricing-section';

type Props = {
  model: GlobalModelResponse;
  groups: BillingGroup[];
  loading: boolean;
  errorMessage?: string;
  title?: string;
};

export function GlobalModelBillingGroupPricing(props: Props) {
  const { t } = useTranslate('admin');
  const state = groupPricingState(props.loading, props.errorMessage, props.groups.length, t);

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{props.title ?? t('models.groupPricing')}</Typography>
      {state}
      {!props.loading && !props.errorMessage ? (
        <Stack spacing={1.5}>
          {props.groups.map((group) => (
            <GroupCard
              key={group.id}
              model={props.model}
              group={group}
            />
          ))}
        </Stack>
      ) : null}
    </Stack>
  );
}

function groupPricingState(
  loading: boolean,
  errorMessage: string | undefined,
  length: number,
  t: (key: string) => string
) {
  if (loading) return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  if (errorMessage) return <Alert severity="error">{errorMessage}</Alert>;
  if (length === 0) return <Typography color="text.secondary">{t('models.groupPricingEmpty')}</Typography>;
  return null;
}

function GroupCard({
  model,
  group,
}: {
  model: GlobalModelResponse;
  group: BillingGroup;
}) {
  const allowed = groupAllowsModel(group, model.id);
  const usable = group.is_active && allowed;

  return (
    <Stack spacing={1.25} sx={panelSx}>
      <GroupHeading group={group} allowed={allowed} />
      <GroupAvailability group={group} allowed={allowed} />
      {usable ? (
        <ModelEffectivePricingSection
          model={model}
          multiplier={group.billing_multiplier}
        />
      ) : null}
    </Stack>
  );
}

function GroupHeading({ group, allowed }: { group: BillingGroup; allowed: boolean }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75}>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
        <Typography variant="subtitle2" noWrap sx={{ flexGrow: 1 }}>
          {group.name}
        </Typography>
        {group.is_system ? <Label variant="soft">{t('common.system')}</Label> : null}
        <Label color={group.is_active ? 'success' : 'default'} variant="soft">
          {group.is_active ? t('common.active') : t('common.disabled')}
        </Label>
        <Label color={allowed ? 'info' : 'default'} variant="soft">
          x{formatMultiplier(group.billing_multiplier)}
        </Label>
      </Stack>
      <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-word' }}>
        {group.description || group.code}
      </Typography>
    </Stack>
  );
}

function GroupAvailability({ group, allowed }: { group: BillingGroup; allowed: boolean }) {
  const { t } = useTranslate('admin');
  if (!group.is_active) return <Alert severity="warning">{t('models.groupInactive')}</Alert>;
  if (!allowed) return <Alert severity="info">{t('models.groupModelDenied')}</Alert>;
  return <Typography variant="caption" color="text.secondary">{t('models.groupModelAllowed')}</Typography>;
}

function groupAllowsModel(group: BillingGroup, modelId: string) {
  return group.allowed_model_ids.length === 0 || group.allowed_model_ids.includes(modelId);
}

function formatMultiplier(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};
