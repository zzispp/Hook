import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ProductCreateView } from 'src/sections/product/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Create a new product | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ProductCreateView />;
}
