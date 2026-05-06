import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { MaintenanceView } from 'src/sections/maintenance/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Maintenance - ${CONFIG.appName}` };

export default function Page() {
  return <MaintenanceView />;
}
