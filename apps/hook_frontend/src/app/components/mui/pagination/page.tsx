import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { PaginationView } from 'src/sections/_examples/mui/pagination-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Pagination | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <PaginationView />;
}
