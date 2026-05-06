import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CenteredUpdatePasswordView } from 'src/auth/view/auth-demo/centered';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: `Update password | Layout centered - ${CONFIG.appName}`,
};

export default function Page() {
  return <CenteredUpdatePasswordView />;
}
