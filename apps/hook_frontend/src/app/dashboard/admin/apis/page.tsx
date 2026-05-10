import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ApiManagementView } from 'src/sections/admin/api-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.apiManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ApiManagementView />;
}
