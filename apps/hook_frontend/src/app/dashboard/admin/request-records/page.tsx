import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { RequestRecordsView } from 'src/sections/admin/request-records-view';

export const metadata: Metadata = { title: `请求记录 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <RequestRecordsView />;
}
