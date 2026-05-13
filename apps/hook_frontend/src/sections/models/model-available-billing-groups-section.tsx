'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';

import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

const MAX_VISIBLE_MODELS = 12;

type Props = {
  groups: BillingGroup[];
  models: GlobalModelResponse[];
  loading: boolean;
  errorMessage?: string;
  onView: (group: BillingGroup) => void;
};

export function ModelAvailableBillingGroupsSection({
  groups,
  models,
  loading,
  errorMessage,
  onView,
}: Props) {
  return (
    <Stack spacing={1.5}>
      <GroupState loading={loading} errorMessage={errorMessage} groups={groups} />
      {!loading && !errorMessage ? (
        <Stack spacing={1.5}>
          {groups.map((group) => (
            <GroupCard key={group.id} group={group} models={models} onView={onView} />
          ))}
        </Stack>
      ) : null}
    </Stack>
  );
}

function GroupState({
  loading,
  errorMessage,
  groups,
}: {
  loading: boolean;
  errorMessage?: string;
  groups: BillingGroup[];
}) {
  const { t } = useTranslate('admin');

  if (loading) return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  if (errorMessage) return <Alert severity="error">{errorMessage}</Alert>;
  if (groups.length === 0) return <Typography color="text.secondary">{t('models.groupPricingEmpty')}</Typography>;
  return null;
}

function GroupCard({
  group,
  models,
  onView,
}: {
  group: BillingGroup;
  models: GlobalModelResponse[];
  onView: (group: BillingGroup) => void;
}) {
  const { t } = useTranslate('admin');
  const modelNames = allowedModelNames(group, models);
  const visibleNames = group.allowed_model_ids.length === 0 ? [] : modelNames.slice(0, MAX_VISIBLE_MODELS);
  const hiddenCount = modelNames.length - visibleNames.length;

  return (
    <Stack spacing={1.25} sx={panelSx}>
      <Stack spacing={0.75}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap sx={{ flexGrow: 1 }}>
            {group.name}
          </Typography>
          {group.is_system ? <Label variant="soft">{t('common.system')}</Label> : null}
          <Label color="info" variant="soft">
            x{formatMultiplier(group.billing_multiplier)}
          </Label>
        </Stack>
        <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-word' }}>
          {group.description || group.code}
        </Typography>
      </Stack>

      <Stack spacing={0.75}>
        <Typography variant="caption" color="text.secondary">
          {t('fields.allowedModels')}
        </Typography>
        {group.allowed_model_ids.length === 0 ? (
          <Typography variant="body2" color="text.secondary">
            {t('billingGroups.allModels')}
          </Typography>
        ) : null}
        {group.allowed_model_ids.length > 0 ? (
          <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
            {visibleNames.map((name) => (
              <Chip key={`${group.id}-${name}`} size="small" label={name} />
            ))}
            {hiddenCount > 0 ? <Chip size="small" label={`+${hiddenCount}`} /> : null}
          </Stack>
        ) : null}
      </Stack>

      <Button
        size="small"
        variant="outlined"
        startIcon={<Iconify icon="solar:eye-bold" />}
        onClick={() => onView(group)}
        sx={{ alignSelf: 'flex-start' }}
      >
        {t('common.details')}
      </Button>
    </Stack>
  );
}

function allowedModelNames(group: BillingGroup, models: GlobalModelResponse[]) {
  const entries = models.map((model) => ({
    id: model.id,
    name: model.display_name || model.name,
  }));
  if (group.allowed_model_ids.length === 0) {
    return entries.map((model) => model.name);
  }

  const labels = new Map(entries.map((model) => [model.id, model.name]));
  return group.allowed_model_ids.map((id) => labels.get(id) ?? id);
}

function formatMultiplier(value: number) {
  return Number.isInteger(value)
    ? String(value)
    : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};
