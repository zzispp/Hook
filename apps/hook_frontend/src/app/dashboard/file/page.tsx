import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OverviewFileView } from 'src/sections/overview/file/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `File | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OverviewFileView />;
}
