import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ScrollbarView } from 'src/sections/_examples/extra/scrollbar-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Scrollbar | Components - ${CONFIG.appName}` };

export default function Page() {
  return <ScrollbarView />;
}
