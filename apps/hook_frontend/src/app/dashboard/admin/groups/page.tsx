import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BillingGroupManagementView } from 'src/sections/admin/billing-group-management-view';

export const metadata: Metadata = { title: `计费分组 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <BillingGroupManagementView />;
}
