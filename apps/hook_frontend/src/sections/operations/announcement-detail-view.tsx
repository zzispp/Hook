'use client';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { fDateTime } from 'src/utils/format-time';

import { useAnnouncement } from 'src/actions/operations';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Markdown } from 'src/components/markdown';

import { AdminBreadcrumbs } from 'src/sections/admin/shared';

import { AnnouncementTypeLabel } from './operation-labels';

export function AnnouncementDetailView({ id }: { id: string }) {
  const announcement = useAnnouncement(id);

  return (
    <DashboardContent maxWidth="md">
      <AdminBreadcrumbs headingCode={DASHBOARD_MENU_CODES.announcements} />
      {announcement.error ? <Alert severity="error">{announcement.error.message}</Alert> : null}
      {announcement.data ? (
        <Stack spacing={3}>
          <Stack spacing={1}>
            <AnnouncementTypeLabel value={announcement.data.announcement_type} />
            <Typography variant="h3">{announcement.data.title}</Typography>
            <Typography variant="caption" color="text.disabled">
              {fDateTime(announcement.data.updated_at)}
            </Typography>
          </Stack>
          <Markdown>{announcement.data.content_markdown}</Markdown>
        </Stack>
      ) : null}
    </DashboardContent>
  );
}
