import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TableView } from 'src/sections/_examples/mui/table-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Table | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TableView />;
}
