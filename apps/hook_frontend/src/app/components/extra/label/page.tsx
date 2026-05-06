import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { LabelView } from 'src/sections/_examples/extra/label-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Label | Components - ${CONFIG.appName}` };

export default function Page() {
  return <LabelView />;
}
