import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { MenuView } from 'src/sections/_examples/mui/menu-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Menu | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <MenuView />;
}
