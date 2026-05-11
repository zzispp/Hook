import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ProviderManagementView } from 'src/sections/admin/provider-management-view';

export const metadata: Metadata = { title: `提供商管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ProviderManagementView />;
}
