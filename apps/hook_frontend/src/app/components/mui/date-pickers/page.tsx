import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { DatePickersView } from 'src/sections/_examples/mui/date-pickers-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Date pickers | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <DatePickersView />;
}
