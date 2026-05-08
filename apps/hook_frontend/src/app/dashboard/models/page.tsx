import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ModelCatalogView } from 'src/sections/models/model-catalog-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Model catalog | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ModelCatalogView />;
}
