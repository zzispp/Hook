'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { UserGroup } from 'src/types/user-group';
import type { GlobalModelResponse } from 'src/types/model';
import type { Provider, ProviderApiKey } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { fDateTime } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { displayUserGroup } from './user-group-utils';

const MAX_VISIBLE_ITEMS = 20;

type Props = {
  group: BillingGroup | null;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  providers: Pick<Provider, 'id' | 'name'>[];
  providerKeysByProvider: Record<string, ProviderApiKey[]>;
  userGroups: UserGroup[];
  open: boolean;
  onClose: () => void;
};

export function BillingGroupDetailDialog({
  group,
  models,
  providers,
  providerKeysByProvider,
  userGroups,
  open,
  onClose,
}: Props) {
  if (!group) return null;

  return (
    <Dialog open={open} fullWidth maxWidth="md" onClose={onClose}>
      <DialogHeader group={group} onClose={onClose} />
      <DialogContent dividers>
        <Stack spacing={2.5}>
          <SummaryGrid group={group} />
          <Description group={group} />
          <Divider />
          <ModelSelectionSection group={group} models={models} />
          <ProviderSelectionSection group={group} providers={providers} />
          <ProviderKeySelectionSection
            group={group}
            providers={providers}
            providerKeysByProvider={providerKeysByProvider}
          />
          <UserGroupSelectionSection group={group} userGroups={userGroups} />
        </Stack>
      </DialogContent>
    </Dialog>
  );
}

function ProviderKeySelectionSection({
  group,
  providers,
  providerKeysByProvider,
}: {
  group: BillingGroup;
  providers: Pick<Provider, 'id' | 'name'>[];
  providerKeysByProvider: Record<string, ProviderApiKey[]>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SelectionSection
      title={t('fields.allowedProviderKeys')}
      summaryLabel={
        group.allowed_provider_key_ids.length === 0
          ? t('billingGroups.allProviderKeys')
          : undefined
      }
      items={namedProviderKeys(group, providers, providerKeysByProvider)}
    />
  );
}

function DialogHeader({ group, onClose }: { group: BillingGroup; onClose: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <DialogTitle sx={titleSx}>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="h6" noWrap>
            {group.name}
          </Typography>
          <Label color={group.is_active ? 'success' : 'default'} variant="soft">
            {group.is_active ? t('common.active') : t('common.disabled')}
          </Label>
          {group.is_system ? <Label variant="soft">{t('common.system')}</Label> : null}
        </Stack>
        <Typography variant="caption" color="text.secondary" sx={codeSx}>
          {group.code}
        </Typography>
      </Box>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </DialogTitle>
  );
}

function Description({ group }: { group: BillingGroup }) {
  if (!group.description) return null;

  return (
    <Typography variant="body2" color="text.secondary">
      {group.description}
    </Typography>
  );
}

function SummaryGrid({ group }: { group: BillingGroup }) {
  const { t } = useTranslate('admin');
  const items = [
    [t('fields.billingMultiplier'), formatMultiplier(group.billing_multiplier)],
    [t('common.sortOrder'), String(group.sort_order)],
    [t('fields.createdAt'), fDateTime(group.created_at)],
    [t('fields.updatedAt'), fDateTime(group.updated_at)],
  ];

  return (
    <Box sx={summaryGridSx}>
      {items.map(([label, value]) => (
        <Stack key={label} spacing={0.5} sx={summaryItemSx}>
          <Typography variant="caption" color="text.secondary">
            {label}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
            {value}
          </Typography>
        </Stack>
      ))}
    </Box>
  );
}

function SelectionSection({
  title,
  summaryLabel,
  items,
}: {
  title: string;
  summaryLabel?: string;
  items: string[];
}) {
  const visibleItems = items.slice(0, MAX_VISIBLE_ITEMS);
  const hiddenCount = items.length - visibleItems.length;

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{title}</Typography>
      {summaryLabel ? (
        <Typography variant="body2" color="text.secondary">
          {summaryLabel}
        </Typography>
      ) : null}
      <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
        {visibleItems.map((item) => (
          <Chip key={`${title}-${item}`} size="small" label={item} />
        ))}
        {hiddenCount > 0 ? <Chip size="small" label={`+${hiddenCount}`} /> : null}
        {!summaryLabel && visibleItems.length === 0 ? (
          <Typography variant="body2" color="text.secondary">
            -
          </Typography>
        ) : null}
      </Stack>
    </Stack>
  );
}

function ModelSelectionSection({
  group,
  models,
}: {
  group: BillingGroup;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <SelectionSection
      title={t('fields.allowedModels')}
      summaryLabel={group.allowed_model_ids.length === 0 ? t('billingGroups.allModels') : undefined}
      items={namedModels(group, models)}
    />
  );
}

function ProviderSelectionSection({
  group,
  providers,
}: {
  group: BillingGroup;
  providers: Pick<Provider, 'id' | 'name'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <SelectionSection
      title={t('fields.allowedProviders')}
      summaryLabel={
        group.allowed_provider_ids.length === 0 ? t('billingGroups.allProviders') : undefined
      }
      items={namedProviders(group, providers)}
    />
  );
}

function UserGroupSelectionSection({
  group,
  userGroups,
}: {
  group: BillingGroup;
  userGroups: UserGroup[];
}) {
  const { t } = useTranslate('admin');

  return (
    <SelectionSection
      title={t('fields.visibleUserGroups')}
      summaryLabel={
        group.visible_user_group_codes.length === 0
          ? t('billingGroups.noVisibleUserGroups')
          : undefined
      }
      items={namedUserGroups(group, userGroups)}
    />
  );
}

function namedModels(
  group: BillingGroup,
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[]
) {
  const labels = new Map(models.map((model) => [model.id, model.display_name || model.name]));
  if (group.allowed_model_ids.length === 0) {
    return models.map((model) => model.display_name || model.name);
  }
  return group.allowed_model_ids.map((id) => labels.get(id) ?? id);
}

function namedProviders(group: BillingGroup, providers: Pick<Provider, 'id' | 'name'>[]) {
  const labels = new Map(providers.map((provider) => [provider.id, provider.name]));
  if (group.allowed_provider_ids.length === 0) {
    return providers.map((provider) => provider.name);
  }
  return group.allowed_provider_ids.map((id) => labels.get(id) ?? id);
}

function namedProviderKeys(
  group: BillingGroup,
  providers: Pick<Provider, 'id' | 'name'>[],
  providerKeysByProvider: Record<string, ProviderApiKey[]>
) {
  const keys = providers.flatMap((provider) =>
    (providerKeysByProvider[provider.id] ?? []).map((key) => [
      key.id,
      `${provider.name} / ${key.name}`,
    ])
  );
  const labels = new Map(keys);
  if (group.allowed_provider_key_ids.length === 0) {
    return Array.from(labels.values());
  }
  return group.allowed_provider_key_ids.map((id) => labels.get(id) ?? id);
}

function namedUserGroups(group: BillingGroup, userGroups: UserGroup[]) {
  return group.visible_user_group_codes.map((code) => displayUserGroup(code, userGroups));
}

function formatMultiplier(value: number) {
  return Number.isInteger(value)
    ? String(value)
    : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}

const titleSx = {
  display: 'flex',
  alignItems: 'flex-start',
  gap: 1,
};

const codeSx = {
  fontFamily: 'monospace',
};

const summaryGridSx = {
  display: 'grid',
  gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, minmax(0, 1fr))' },
  gap: 1.5,
};

const summaryItemSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};
