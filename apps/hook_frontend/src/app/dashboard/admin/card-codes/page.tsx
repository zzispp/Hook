import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminCardCodeManagementView } from 'src/sections/card-code/admin-card-code-management-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.cardCodeManagement);
}

export default function Page() {
  return <AdminCardCodeManagementView />;
}
