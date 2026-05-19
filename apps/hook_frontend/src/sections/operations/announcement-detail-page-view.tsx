'use client';

import { useSearchParams } from 'next/navigation';

import Alert from '@mui/material/Alert';

import { DashboardContent } from 'src/layouts/dashboard';

import { AnnouncementDetailView } from './announcement-detail-view';

export function AnnouncementDetailPageView() {
  const id = useSearchParams().get('id')?.trim() ?? '';

  if (!id) {
    return (
      <DashboardContent maxWidth="md">
        <Alert severity="error">Missing announcement id.</Alert>
      </DashboardContent>
    );
  }

  return <AnnouncementDetailView id={id} />;
}
