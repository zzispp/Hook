import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { IconifyView } from 'src/sections/_examples/foundation/icons-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Iconify Icon | Foundations - ${CONFIG.appName}` };

export default function Page() {
  return <IconifyView />;
}
