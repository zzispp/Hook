import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminAffiliateManagementView } from 'src/sections/affiliate/admin-affiliate-management-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.affiliateManagement);
}

export default function Page() {
  return <AdminAffiliateManagementView />;
}
