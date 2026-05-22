import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ScheduledTasksView } from 'src/sections/admin/scheduled-tasks-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.scheduledTaskManagement);
}

export default function Page() {
  return <ScheduledTasksView />;
}
