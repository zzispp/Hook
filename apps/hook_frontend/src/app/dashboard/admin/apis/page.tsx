import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ApiManagementView } from 'src/sections/admin/api-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `API管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ApiManagementView />;
}
