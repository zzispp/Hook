import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { UserManagementView } from 'src/sections/admin/user-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `用户管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <UserManagementView />;
}
