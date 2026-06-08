import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { MaintenanceView } from 'src/sections/maintenance/view';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('maintenance');
}

export default function Page() {
  return <MaintenanceView />;
}
