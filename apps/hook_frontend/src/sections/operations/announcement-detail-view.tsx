'use client';

import Alert from '@mui/material/Alert';

import { useAnnouncement } from 'src/actions/operations';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AnnouncementContent } from './announcement-content';

export function AnnouncementDetailView({ id }: { id: string }) {
  const announcement = useAnnouncement(id);

  return (
    <DashboardContent maxWidth="md">
      <AdminBreadcrumbs headingCode={DASHBOARD_MENU_CODES.announcements} />
      {announcement.error ? <Alert severity="error">{announcement.error.message}</Alert> : null}
      {announcement.data ? (
        <AnnouncementContent announcement={announcement.data} />
      ) : null}
    </DashboardContent>
  );
}
