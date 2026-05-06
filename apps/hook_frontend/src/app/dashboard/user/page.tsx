import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { UserProfileView } from 'src/sections/user/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `User profile | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <UserProfileView />;
}
