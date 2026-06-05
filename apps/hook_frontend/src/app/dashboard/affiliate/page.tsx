import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AffiliateCenterView } from 'src/sections/account-affiliate/affiliate-center-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.affiliateCenter);
}

export default function Page() {
  return <AffiliateCenterView />;
}
