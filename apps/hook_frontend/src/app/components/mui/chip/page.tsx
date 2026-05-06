import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ChipView } from 'src/sections/_examples/mui/chip-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Chip | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <ChipView />;
}
