import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BlankView } from 'src/sections/blank/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Blank | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <BlankView />;
}
