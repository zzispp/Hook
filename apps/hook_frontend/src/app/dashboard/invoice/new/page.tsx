import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { InvoiceCreateView } from 'src/sections/invoice/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Create a new invoice | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <InvoiceCreateView />;
}
