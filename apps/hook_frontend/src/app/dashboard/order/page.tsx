import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OrderListView } from 'src/sections/order/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Order list | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OrderListView />;
}
