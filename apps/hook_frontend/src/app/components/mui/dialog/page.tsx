import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { DialogView } from 'src/sections/_examples/mui/dialog-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Dialog | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <DialogView />;
}
