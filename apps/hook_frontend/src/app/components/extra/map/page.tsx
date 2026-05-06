import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { MapView } from 'src/sections/_examples/extra/map-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Map | Components - ${CONFIG.appName}` };

export default function Page() {
  return <MapView />;
}
