import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { GridView } from 'src/sections/_examples/foundation/grid-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Grid | Foundations - ${CONFIG.appName}` };

export default function Page() {
  return <GridView />;
}
