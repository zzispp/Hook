import type { TFunction } from 'i18next';
import type { ModelStatusValue } from 'src/types/model-status';

import { Label } from 'src/components/label';

import { statusLabel } from './model-status-timeline';

export function StatusLabel({ status, t }: { status?: ModelStatusValue | null; t: TFunction<'admin'> }) {
  return (
    <Label color={statusColor(status)} variant="soft">
      {statusLabel(status, t)}
    </Label>
  );
}

function statusColor(status?: ModelStatusValue | null) {
  if (status === 'operational') return 'success';
  if (status === 'degraded') return 'warning';
  if (status === 'failed' || status === 'error') return 'error';
  return 'default';
}
