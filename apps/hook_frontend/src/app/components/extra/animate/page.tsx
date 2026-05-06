import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AnimateView } from 'src/sections/_examples/extra/animate-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Animate | Components - ${CONFIG.appName}` };

export default function Page() {
  return <AnimateView />;
}
