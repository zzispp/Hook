import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ModelCatalogView } from 'src/sections/models/model-catalog-view';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.modelCatalog);
}

export default function Page() {
  return <ModelCatalogView />;
}
