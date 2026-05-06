import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { IconsView } from 'src/sections/_examples/foundation/icons-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Icons | Foundations - ${CONFIG.appName}` };

export default function Page() {
  return <IconsView />;
}
