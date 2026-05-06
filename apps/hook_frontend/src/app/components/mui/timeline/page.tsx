import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TimelineView } from 'src/sections/_examples/mui/timeline-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Timeline | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TimelineView />;
}
