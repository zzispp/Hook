import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { BillingGroupManagementView } from 'src/sections/admin/billing-group-management-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.billingGroups);
}

export default function Page() {
  return <BillingGroupManagementView />;
}
