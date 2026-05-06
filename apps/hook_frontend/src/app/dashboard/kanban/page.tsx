import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { KanbanView } from 'src/sections/kanban/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Kanban | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <KanbanView />;
}
