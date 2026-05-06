import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ImageView } from 'src/sections/_examples/extra/image-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Image | Components - ${CONFIG.appName}` };

export default function Page() {
  return <ImageView />;
}
