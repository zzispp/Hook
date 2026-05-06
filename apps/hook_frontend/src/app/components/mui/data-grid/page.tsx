import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { DataGridView } from 'src/sections/_examples/mui/data-grid-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `DataGrid | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <DataGridView />;
}
