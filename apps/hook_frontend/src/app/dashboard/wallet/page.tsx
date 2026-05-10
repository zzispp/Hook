import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { WalletCenterView } from 'src/sections/wallet/wallet-center-view';

export const metadata: Metadata = { title: `钱包中心 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <WalletCenterView />;
}
