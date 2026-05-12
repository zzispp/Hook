'use client';

import type { Theme } from '@mui/material/styles';
import type { ProviderApiKey } from 'src/types/provider';
import type { IconifyProps } from 'src/components/iconify';
import type { useProviderChildDialogs } from './provider-management-state';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';

export function ProviderApiKeysSection({
  items,
  loading,
  dialogs,
}: {
  items: ProviderApiKey[];
  loading: boolean;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={panelSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
        <Typography variant="subtitle2">{t('providers.keyManagement')}</Typography>
        <Button
          color="inherit"
          size="small"
          variant="outlined"
          startIcon={<Iconify icon="mingcute:add-line" width={14} />}
          onClick={dialogs.openCreateApiKey}
        >
          {t('actions.addProviderKey')}
        </Button>
      </Stack>
      <Box sx={listSx}>
        {items.map((apiKey) => (
          <ApiKeyRow key={apiKey.id} apiKey={apiKey} dialogs={dialogs} />
        ))}
        <EmptyList loading={loading} length={items.length} />
      </Box>
    </Box>
  );
}

function ApiKeyRow({
  apiKey,
  dialogs,
}: {
  apiKey: ProviderApiKey;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={[rowSx, !apiKey.is_active && inactiveRowSx]}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Iconify icon="custom:drag-dots-fill" width={16} sx={{ color: 'text.disabled', flexShrink: 0 }} />
          <Box sx={{ minWidth: 0 }}>
            <Stack direction="row" alignItems="center" spacing={0.75}>
              <Typography
                noWrap
                variant="body2"
                title={t('providers.copyKeyName')}
                onClick={() => copyKeyName(apiKey.name, t)}
                sx={nameSx}
              >
                {apiKey.name}
              </Typography>
            </Stack>
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
        </Stack>
        <Stack direction="row" alignItems="center" spacing={0.25} sx={{ flexShrink: 0 }}>
          <KeyActionButton title={t('common.edit')} icon="solar:pen-bold" onClick={() => dialogs.openEditApiKey(apiKey)} />
          <KeyActionButton
            title={apiKey.is_active ? t('providers.disableKey') : t('providers.enableKey')}
            icon="ic:round-power-settings-new"
            onClick={() => dialogs.toggleApiKey(apiKey)}
          />
          <KeyActionButton
            title={t('providers.deleteKey')}
            icon="solar:trash-bin-trash-bold"
            onClick={() => dialogs.setDeletingApiKey(apiKey)}
          />
        </Stack>
      </Stack>
      <Stack direction="row" alignItems="center" spacing={0.75} sx={metaSx}>
        <Typography component="span" variant="caption" sx={prioritySx}>
          P{apiKey.internal_priority}
        </Typography>
        {apiKey.time_range_enabled ? (
          <>
            <MetaDivider />
            <Typography component="span" variant="caption">
              {timeRangeText(apiKey)}
            </Typography>
          </>
        ) : null}
      </Stack>
    </Box>
  );
}

function KeyActionButton({ title, icon, onClick }: { title: string; icon: IconifyProps['icon']; onClick: () => void }) {
  return (
    <Tooltip title={title}>
      <IconButton size="small" sx={actionButtonSx} onClick={onClick}>
        <Iconify icon={icon} width={14} />
      </IconButton>
    </Tooltip>
  );
}

function MetaDivider() {
  return <Box component="span" sx={{ color: 'text.disabled' }}>|</Box>;
}

async function copyKeyName(name: string, t: (key: string) => string) {
  try {
    await navigator.clipboard.writeText(name);
    toast.success(t('messages.apiKeyCopied'));
  } catch {
    toast.error(t('messages.copyFailed'));
  }
}

function timeRangeText(apiKey: ProviderApiKey) {
  const start = apiKey.time_range_start || '--:--';
  const end = apiKey.time_range_end || '--:--';
  return `${start}-${end}`;
}

const panelSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 2, overflow: 'hidden' };
const headerSx = { p: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const listSx = { '& > * + *': { borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` } };
const rowSx = { px: 2, py: 1.25, transition: (theme: Theme) => theme.transitions.create('background-color'), '&:hover': { bgcolor: 'action.hover' } };
const inactiveRowSx = { opacity: 0.52, bgcolor: 'action.disabledBackground' };
const nameSx = { fontWeight: 600, cursor: 'pointer', '&:hover': { color: 'primary.main' } };
const maskedKeySx = { fontFamily: 'monospace', color: 'text.secondary', fontSize: 11 };
const tinyButtonSx = { width: 16, height: 16, color: 'text.secondary' };
const actionButtonSx = { width: 28, height: 28 };
const metaSx = { mt: 0.5, color: 'text.secondary', fontSize: 11, flexWrap: 'wrap' };
const prioritySx = { color: 'text.primary', fontWeight: 600, cursor: 'default' };
