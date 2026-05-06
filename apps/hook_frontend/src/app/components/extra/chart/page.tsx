import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ChartView } from 'src/sections/_examples/extra/chart-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Chart | Components - ${CONFIG.appName}` };

export default function Page() {
  return <ChartView />;
}
