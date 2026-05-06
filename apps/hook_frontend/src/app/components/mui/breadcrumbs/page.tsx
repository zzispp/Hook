import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BreadcrumbsView } from 'src/sections/_examples/mui/breadcrumbs-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Breadcrumbs | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <BreadcrumbsView />;
}
