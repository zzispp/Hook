import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { LightboxView } from 'src/sections/_examples/extra/lightbox-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Lightbox | Components - ${CONFIG.appName}` };

export default function Page() {
  return <LightboxView />;
}
