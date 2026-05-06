import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OrganizationalChartView } from 'src/sections/_examples/extra/organizational-chart-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: `Organizational chart | Components - ${CONFIG.appName}`,
};

export default function Page() {
  return <OrganizationalChartView />;
}
