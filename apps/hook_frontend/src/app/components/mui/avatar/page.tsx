import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AvatarView } from 'src/sections/_examples/mui/avatar-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Avatar | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <AvatarView />;
}
