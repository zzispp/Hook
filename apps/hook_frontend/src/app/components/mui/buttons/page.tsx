import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ButtonsView } from 'src/sections/_examples/mui/button-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Buttons | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <ButtonsView />;
}
