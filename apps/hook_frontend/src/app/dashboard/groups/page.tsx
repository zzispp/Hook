import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BillingGroupCatalogView } from 'src/sections/models/billing-group-catalog-view';

export const metadata: Metadata = { title: `价格分组 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <BillingGroupCatalogView />;
}
