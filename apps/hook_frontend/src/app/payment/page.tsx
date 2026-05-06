import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { PaymentView } from 'src/sections/payment/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Payment - ${CONFIG.appName}` };

export default function Page() {
  return <PaymentView />;
}
