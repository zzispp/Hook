import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { UtilitiesView } from 'src/sections/_examples/extra/utilities-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Utilities | Components - ${CONFIG.appName}` };

export default function Page() {
  return <UtilitiesView />;
}
