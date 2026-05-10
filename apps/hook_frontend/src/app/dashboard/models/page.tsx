import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { DASHBOARD_MENU_TITLES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ModelCatalogView } from 'src/sections/models/model-catalog-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `${DASHBOARD_MENU_TITLES.modelCatalog} | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <ModelCatalogView />;
}
