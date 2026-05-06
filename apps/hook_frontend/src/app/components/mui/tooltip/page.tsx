import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TooltipView } from 'src/sections/_examples/mui/tooltip-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Tooltip | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TooltipView />;
}
