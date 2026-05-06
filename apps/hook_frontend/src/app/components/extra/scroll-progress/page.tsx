import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ScrollProgressView } from 'src/sections/_examples/extra/scroll-progress-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Scroll progress | Components - ${CONFIG.appName}` };

export default function Page() {
  return <ScrollProgressView />;
}
