import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ModelCatalogView } from 'src/sections/models/model-catalog-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `模型目录 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ModelCatalogView />;
}
