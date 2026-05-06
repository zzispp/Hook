import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ProductListView } from 'src/sections/product/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Product list | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ProductListView />;
}
