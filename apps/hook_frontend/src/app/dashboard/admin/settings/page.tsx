import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SystemSettingsView } from 'src/sections/admin/system-settings-view';

export const metadata: Metadata = { title: `System Settings | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <SystemSettingsView />;
}
