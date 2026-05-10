import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ModelManagementView } from 'src/sections/admin/model-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.modelManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ModelManagementView />;
}
