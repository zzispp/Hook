import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { BillingGroupManagementView } from 'src/sections/admin/billing-group-management-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.billingGroups} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <BillingGroupManagementView />;
}
