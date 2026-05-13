import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { BillingGroupCatalogView } from 'src/sections/models/billing-group-catalog-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.billingGroupCatalog);
}

export default function Page() {
  return <BillingGroupCatalogView />;
}
