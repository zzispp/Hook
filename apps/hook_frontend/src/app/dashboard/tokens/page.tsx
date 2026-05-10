import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ApiTokenManagementView } from 'src/sections/api-tokens/api-token-management-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.apiTokens} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ApiTokenManagementView />;
}
