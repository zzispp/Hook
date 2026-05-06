import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ShadowsView } from 'src/sections/_examples/foundation/shadows-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Shadows | Foundations - ${CONFIG.appName}` };

export default function Page() {
  return <ShadowsView />;
}
