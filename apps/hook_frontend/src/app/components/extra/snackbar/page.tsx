import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SnackbarView } from 'src/sections/_examples/extra/snackbar-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Snackbar | Components - ${CONFIG.appName}` };

export default function Page() {
  return <SnackbarView />;
}
