import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { MenuManagementView } from 'src/sections/admin/menu-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.menuManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <MenuManagementView />;
}
