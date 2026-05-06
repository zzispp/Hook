import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CalendarView } from 'src/sections/calendar/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Calendar | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <CalendarView />;
}
