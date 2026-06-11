'use client';

import type { ChipProps } from '@mui/material/Chip';
import type { ProviderQuickImportSyncStatus , ProviderQuickImportTokenPreview } from 'src/types/provider-quick-import';

import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

export function QuickImportResolutionHeader({
  title,
  subtitle,
  onClose,
}: {
  title: string;
  subtitle?: string;
  onClose: () => void;
}) {
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <Stack sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6">{title}</Typography>
        {subtitle ? <Typography variant="caption" color="text.secondary">{subtitle}</Typography> : null}
      </Stack>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

export function QuickImportResolutionStatusChips({ statuses }: { statuses: ProviderQuickImportSyncStatus[] }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
      {statuses.map((status) => (
        <Chip key={status} size="small" color={statusColor(status)} variant="soft" label={t(`providers.quickImportSyncStatus.${status}`)} />
      ))}
    </Stack>
  );
}

export function QuickImportResolutionTokenSummary({ token }: { token?: ProviderQuickImportTokenPreview }) {
  if (!token) return null;
  return (
    <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
      <Chip size="small" variant="soft" color="info" label={token.group ?? '-'} />
      <Chip size="small" variant="soft" color="warning" label={`${token.group_ratio}x`} />
      <Chip size="small" variant="outlined" label={token.masked_key} />
    </Stack>
  );
}

export function QuickImportResolutionLoadingState() {
  return (
    <Stack alignItems="center" justifyContent="center" sx={{ py: 6 }}>
      <CircularProgress size={24} />
    </Stack>
  );
}

function statusColor(status: ProviderQuickImportSyncStatus): ChipProps['color'] {
  if (status === 'ok') return 'success';
  if (
    status === 'cost_pending_update' ||
    status === 'cost_unavailable' ||
    status === 'model_candidate_available' ||
    status === 'upstream_key_unavailable'
  ) return 'warning';
  if (status === 'sync_disabled') return 'default';
  if (status === 'source_not_configured') return 'info';
  return 'error';
}
