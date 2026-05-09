import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BillingGroupManagementView } from 'src/sections/admin/billing-group-management-view';

export const metadata: Metadata = { title: `Billing groups | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <BillingGroupManagementView />;
}
