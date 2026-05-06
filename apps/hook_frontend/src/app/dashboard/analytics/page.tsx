import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OverviewAnalyticsView } from 'src/sections/overview/analytics/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Analytics | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OverviewAnalyticsView />;
}
