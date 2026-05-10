import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { SystemSettingsView } from 'src/sections/admin/system-settings-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.systemSettings} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <SystemSettingsView />;
}
