import type { ChipProps } from '@mui/material/Chip';
import type { ProviderQuickImportSyncStatus } from 'src/types/provider-quick-import';

export function quickImportSyncStatusColor(
  status: ProviderQuickImportSyncStatus
): ChipProps['color'] {
  if (status === 'ok') return 'success';
  if (
    status === 'cost_pending_update' ||
    status === 'cost_unavailable' ||
    status === 'model_candidate_available' ||
    status === 'upstream_key_unavailable'
  )
    return 'warning';
  if (status === 'sync_disabled') return 'default';
  if (status === 'source_not_configured') return 'info';
  return 'error';
}

export function hasHardQuickImportStatus(statuses: ProviderQuickImportSyncStatus[]) {
  return statuses.some((status) =>
    [
      'source_fetch_failed',
      'upstream_token_deleted',
      'upstream_token_disabled',
      'upstream_group_removed',
      'upstream_group_changed',
      'upstream_model_removed',
      'no_associated_models',
    ].includes(status)
  );
}
