import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TabsView } from 'src/sections/_examples/mui/tabs-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Tabs | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TabsView />;
}
