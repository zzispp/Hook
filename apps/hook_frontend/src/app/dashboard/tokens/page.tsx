import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ApiTokenManagementView } from 'src/sections/api-tokens/api-token-management-view';

export const metadata: Metadata = { title: `API tokens | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ApiTokenManagementView />;
}
