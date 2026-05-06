import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AccordionView } from 'src/sections/_examples/mui/accordion-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Accordion | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <AccordionView />;
}
