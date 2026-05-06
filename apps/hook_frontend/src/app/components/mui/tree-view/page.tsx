import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TreeView } from 'src/sections/_examples/mui/tree-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Tree view | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TreeView />;
}
