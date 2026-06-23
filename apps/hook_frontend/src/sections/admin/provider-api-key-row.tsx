'use client';

import type { Theme } from '@mui/material/styles';
import type { ProviderApiKey } from 'src/types/provider';
import type { useProviderChildDialogs } from './provider-management-state';
import type { ProviderQuickImportSyncStatus } from 'src/types/provider-quick-import';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { formatApiFormat } from './provider-management-utils';
import { MetaDivider, KeyActionButton } from './provider-api-key-row-parts';
import {
  hasHardQuickImportStatus,
  quickImportSyncStatusColor,
} from './provider-quick-import-status-utils';

type Props = {
  apiKey: ProviderApiKey;
  groupNames: string[];
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  onResolveQuickImportKey: (apiKey: ProviderApiKey) => void;
  onManageKeyModels: (apiKey: ProviderApiKey) => void;
  onAssociateGroups: (apiKey: ProviderApiKey) => void;
};

export function ProviderApiKeyRow(props: Props) {
  const {
    apiKey,
    groupNames,
    dialogs,
    onResolveQuickImportKey,
    onManageKeyModels,
    onAssociateGroups,
  } = props;
  const hardAnomaly = hasHardQuickImportStatus(apiKey.quick_import_sync?.statuses ?? []);

  return (
    <Box sx={[rowSx, !apiKey.is_active && inactiveRowSx]}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Iconify
            icon="custom:drag-dots-fill"
            width={16}
            sx={{ color: 'text.disabled', flexShrink: 0 }}
          />
          <KeyIdentity apiKey={apiKey} />
        </Stack>
        <KeyActions
          apiKey={apiKey}
          hardAnomaly={hardAnomaly}
          dialogs={dialogs}
          onAssociateGroups={onAssociateGroups}
          onResolveQuickImportKey={onResolveQuickImportKey}
          onManageKeyModels={onManageKeyModels}
        />
      </Stack>
      <KeyMeta apiKey={apiKey} groupNames={groupNames} />
    </Box>
  );
}

function KeyIdentity({ apiKey }: { apiKey: ProviderApiKey }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ minWidth: 0 }}>
      <Typography
        noWrap
        variant="body2"
        title={t('providers.copyKeyName')}
        onClick={() => copyKeyName(apiKey.name, t)}
        sx={nameSx}
      >
        {apiKey.name}
      </Typography>
      <Stack direction="row" alignItems="center" spacing={0.5}>
        <Typography variant="caption" sx={maskedKeySx}>
          {apiKey.has_api_key ? t('providers.keyMasked') : '-'}
        </Typography>
        <Tooltip title={t('providers.copyKeyName')}>
          <IconButton size="small" sx={tinyButtonSx} onClick={() => copyKeyName(apiKey.name, t)}>
            <Iconify icon="solar:copy-bold" width={10} />
          </IconButton>
        </Tooltip>
      </Stack>
    </Box>
  );
}

type KeyActionsProps = Pick<
  Props,
  | 'apiKey'
  | 'dialogs'
  | 'onAssociateGroups'
  | 'onResolveQuickImportKey'
  | 'onManageKeyModels'
> & {
  hardAnomaly: boolean;
};

function KeyActions({
  apiKey,
  hardAnomaly,
  dialogs,
  onAssociateGroups,
  onResolveQuickImportKey,
  onManageKeyModels,
}: KeyActionsProps) {
  const { t } = useTranslate('admin');
  const powerTitle =
    hardAnomaly && !apiKey.is_active
      ? t('providers.quickImportEnableBlocked')
      : apiKey.is_active
        ? t('providers.disableKey')
        : t('providers.enableKey');

  return (
    <Stack direction="row" alignItems="center" spacing={0.25} sx={{ flexShrink: 0 }}>
      {apiKey.quick_import_sync ? (
        <>
          {hardAnomaly ? (
            <KeyActionButton
              title={t('providers.quickImportResolveKey')}
              icon="solar:restart-bold"
              onClick={() => onResolveQuickImportKey(apiKey)}
            />
          ) : null}
          <KeyActionButton
            title={t('providers.keyModelMappingsTitle')}
            icon="solar:settings-bold"
            onClick={() => onManageKeyModels(apiKey)}
          />
        </>
      ) : null}
      <KeyActionButton
        title={t('actions.associateProviderKeyGroups')}
        icon="eva:link-2-fill"
        onClick={() => onAssociateGroups(apiKey)}
      />
      <KeyActionButton
        title={t('common.edit')}
        icon="solar:pen-bold"
        onClick={() => dialogs.openEditApiKey(apiKey)}
      />
      <KeyActionButton
        title={powerTitle}
        icon="ic:round-power-settings-new"
        disabled={hardAnomaly && !apiKey.is_active}
        onClick={() => dialogs.toggleApiKey(apiKey)}
      />
      <KeyActionButton
        title={t('providers.deleteKey')}
        icon="solar:trash-bin-trash-bold"
        onClick={() => dialogs.setDeletingApiKey(apiKey)}
      />
    </Stack>
  );
}

function KeyMeta({ apiKey, groupNames }: { apiKey: ProviderApiKey; groupNames: string[] }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" alignItems="center" spacing={0.75} sx={metaSx}>
      <Typography component="span" variant="caption" sx={prioritySx}>
        P{apiKey.internal_priority}
      </Typography>
      <MetaDivider />
      <Typography component="span" variant="caption">
        {formatList(apiKey.api_formats, t('providers.noSupportedFormats'))}
      </Typography>
      <MetaDivider />
      <Typography component="span" variant="caption">
        {modelPermissionText(apiKey.allowed_model_ids, t)}
      </Typography>
      {apiKey.time_range_enabled ? <TimeRangeMeta apiKey={apiKey} /> : null}
      <KeyGroupChips groupNames={groupNames} />
      <QuickImportSyncChips apiKey={apiKey} />
    </Stack>
  );
}

function TimeRangeMeta({ apiKey }: { apiKey: ProviderApiKey }) {
  const start = apiKey.time_range_start || '--:--';
  const end = apiKey.time_range_end || '--:--';
  return (
    <>
      <MetaDivider />
      <Typography component="span" variant="caption">
        {start}-{end}
      </Typography>
    </>
  );
}

function QuickImportSyncChips({ apiKey }: { apiKey: ProviderApiKey }) {
  const { t } = useTranslate('admin');
  const sync = apiKey.quick_import_sync;
  if (!sync) return null;
  const statuses =
    sync.statuses.length > 0 ? sync.statuses : (['ok'] as ProviderQuickImportSyncStatus[]);

  return (
    <>
      <MetaDivider />
      {statuses.map((status) => (
        <Chip
          key={status}
          size="small"
          color={quickImportSyncStatusColor(status)}
          variant="soft"
          label={t(`providers.quickImportSyncStatus.${status}`)}
          sx={keyGroupChipSx}
        />
      ))}
    </>
  );
}

function KeyGroupChips({ groupNames }: { groupNames: string[] }) {
  if (groupNames.length === 0) return null;
  return (
    <>
      <MetaDivider />
      {groupNames.map((name) => (
        <Chip key={name} size="small" variant="outlined" label={name} sx={keyGroupChipSx} />
      ))}
    </>
  );
}

async function copyKeyName(name: string, t: (key: string) => string) {
  try {
    await navigator.clipboard.writeText(name);
    toast.success(t('messages.apiKeyCopied'));
  } catch {
    toast.error(t('messages.copyFailed'));
  }
}

function formatList(values: string[], emptyText: string) {
  if (!values.length) return emptyText;
  return values.map(formatApiFormat).join(', ');
}

function modelPermissionText(
  values: string[],
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (!values.length) return t('providers.allModels');
  return t('providers.selectedModelCount', { count: values.length });
}

const rowSx = {
  px: 2,
  py: 1.25,
  transition: (theme: Theme) => theme.transitions.create('background-color'),
  '&:hover': { bgcolor: 'action.hover' },
};
const inactiveRowSx = { opacity: 0.52, bgcolor: 'action.disabledBackground' };
const nameSx = { fontWeight: 600, cursor: 'pointer', '&:hover': { color: 'primary.main' } };
const maskedKeySx = { fontFamily: 'monospace', color: 'text.secondary', fontSize: 11 };
const tinyButtonSx = { width: 16, height: 16, color: 'text.secondary' };
const metaSx = { mt: 0.5, color: 'text.secondary', fontSize: 11, flexWrap: 'wrap' };
const prioritySx = { color: 'text.primary', fontWeight: 600, cursor: 'default' };
const keyGroupChipSx = { height: 20, '& .MuiChip-label': { px: 0.75, fontSize: 11 } };
