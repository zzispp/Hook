import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminApiTokenManagementView } from 'src/sections/api-tokens/api-token-management-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.tokenManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminApiTokenManagementView />;
}
