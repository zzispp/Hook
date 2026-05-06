import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { DrawerView } from 'src/sections/_examples/mui/drawer-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Drawer | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <DrawerView />;
}
