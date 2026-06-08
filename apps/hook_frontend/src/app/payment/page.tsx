import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { PaymentView } from 'src/sections/payment/view';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('payment');
}

export default function Page() {
  return <PaymentView />;
}
