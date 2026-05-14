import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AnnouncementDetailView } from 'src/sections/operations';

type Props = {
  params: Promise<{ id: string }>;
};

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.announcements);
}

export default async function Page({ params }: Props) {
  const { id } = await params;

  return <AnnouncementDetailView id={id} />;
}
