import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { WalletCenterView } from 'src/sections/wallet/wallet-center-view';

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.walletCenter} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <WalletCenterView />;
}
