import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OverviewAppView } from 'src/sections/overview/app/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OverviewAppView />;
}
