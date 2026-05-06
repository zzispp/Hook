import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SwitchView } from 'src/sections/_examples/mui/switch-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Switch | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <SwitchView />;
}
