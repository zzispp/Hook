import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { ProfileView } from 'src/sections/profile/profile-view';

export const metadata: Metadata = { title: `Profile | ${CONFIG.appName}` };

export default function Page() {
  return <ProfileView />;
}
