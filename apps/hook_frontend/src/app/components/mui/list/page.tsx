import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ListView } from 'src/sections/_examples/mui/list-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `List | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <ListView />;
}
