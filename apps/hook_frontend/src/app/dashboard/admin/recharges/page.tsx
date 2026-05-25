import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminRechargeManagementView } from 'src/sections/recharge/admin-recharge-management-view';

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.rechargeManagement);
}

export default function Page() {
  return <AdminRechargeManagementView />;
}

