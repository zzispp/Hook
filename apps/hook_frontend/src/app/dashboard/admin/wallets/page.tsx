import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { AdminWalletManagementView } from 'src/sections/wallet/admin-wallet-management-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.walletManagement} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminWalletManagementView />;
}
