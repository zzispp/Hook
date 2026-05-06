import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OverviewBookingView } from 'src/sections/overview/booking/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Booking | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OverviewBookingView />;
}
