import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { FaqsView } from 'src/sections/faqs/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Faqs - ${CONFIG.appName}` };

export default function Page() {
  return <FaqsView />;
}
