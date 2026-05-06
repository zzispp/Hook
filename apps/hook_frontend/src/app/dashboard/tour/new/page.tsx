import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TourCreateView } from 'src/sections/tour/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Create a new tour | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <TourCreateView />;
}
