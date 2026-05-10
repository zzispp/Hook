import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { UserManagementView } from 'src/sections/admin/user-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.userManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <UserManagementView />;
}
