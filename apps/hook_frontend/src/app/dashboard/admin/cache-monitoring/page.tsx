import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { CacheMonitoringView } from 'src/sections/admin/cache-monitoring-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.cacheMonitoring);
}

export default function Page() {
  return <CacheMonitoringView />;
}
