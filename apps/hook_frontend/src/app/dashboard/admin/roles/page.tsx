import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { RoleManagementView } from 'src/sections/admin/role-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.roleManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <RoleManagementView />;
}
