import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { PricingView } from 'src/sections/pricing/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Pricing - ${CONFIG.appName}` };

export default function Page() {
  return <PricingView />;
}
