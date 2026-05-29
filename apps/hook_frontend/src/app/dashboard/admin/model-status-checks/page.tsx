import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ModelStatusChecksView } from 'src/sections/admin/model-status-checks-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.modelStatusChecks);
}

export default function Page() {
  return <ModelStatusChecksView />;
}
