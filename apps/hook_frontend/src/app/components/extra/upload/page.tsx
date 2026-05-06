import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { UploadView } from 'src/sections/_examples/extra/upload-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Upload | Components - ${CONFIG.appName}` };

export default function Page() {
  return <UploadView />;
}
