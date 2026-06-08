import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { PricingView } from 'src/sections/pricing/view';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('pricing');
}

export default function Page() {
  return <PricingView />;
}
