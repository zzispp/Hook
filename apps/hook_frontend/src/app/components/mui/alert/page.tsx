import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AlertView } from 'src/sections/_examples/mui/alert-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Alert | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <AlertView />;
}
