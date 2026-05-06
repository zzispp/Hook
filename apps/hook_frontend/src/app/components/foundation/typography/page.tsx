import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TypographyView } from 'src/sections/_examples/foundation/typography-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Typography | Foundations - ${CONFIG.appName}` };

export default function Page() {
  return <TypographyView />;
}
