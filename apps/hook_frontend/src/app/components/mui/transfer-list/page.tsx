import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TransferListView } from 'src/sections/_examples/mui/transfer-list-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Transfer list | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TransferListView />;
}
