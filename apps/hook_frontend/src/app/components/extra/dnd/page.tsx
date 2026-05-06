import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { DndView } from 'src/sections/_examples/extra/dnd-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Dnd | Components - ${CONFIG.appName}` };

export default function Page() {
  return <DndView />;
}
