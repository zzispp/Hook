import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ModelManagementView } from 'src/sections/admin/model-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `模型管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ModelManagementView />;
}
