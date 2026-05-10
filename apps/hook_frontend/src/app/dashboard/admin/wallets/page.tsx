import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AdminWalletManagementView } from 'src/sections/wallet/admin-wallet-management-view';

export const metadata: Metadata = { title: `钱包管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminWalletManagementView />;
}
