import type { TicketStatus, TicketPriority, AnnouncementType } from 'src/types/operations';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

export function AnnouncementTypeLabel({ value }: { value: AnnouncementType }) {
  const { t } = useTranslate('admin');
  const color =
    (value === 'important' && 'error') ||
    (value === 'maintenance' && 'warning') ||
    (value === 'warning' && 'warning') ||
    'info';

  return (
    <Label color={color} variant="soft">
      {t(`operations.announcement.types.${value}`)}
    </Label>
  );
}

export function TicketStatusLabel({ value }: { value: TicketStatus }) {
  const { t } = useTranslate('admin');
  const color =
    (value === 'closed' && 'default') ||
    (value === 'resolved' && 'success') ||
    (value === 'waiting_user' && 'warning') ||
    (value === 'in_progress' && 'info') ||
    'error';

  return (
    <Label color={color} variant="soft">
      {t(`operations.ticket.status.${value}`)}
    </Label>
  );
}

export function TicketPriorityLabel({ value }: { value: TicketPriority }) {
  const { t } = useTranslate('admin');
  const color = (value === 'urgent' && 'error') || (value === 'high' && 'warning') || 'default';

  return (
    <Label color={color} variant="soft">
      {t(`operations.ticket.priority.${value}`)}
    </Label>
  );
}
