import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AdminApiTokenManagementView } from 'src/sections/api-tokens/api-token-management-view';

export const metadata: Metadata = { title: `Token management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminApiTokenManagementView />;
}
