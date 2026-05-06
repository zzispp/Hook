import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CenteredSignInView } from 'src/auth/view/auth-demo/centered';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | Layout centered - ${CONFIG.appName}` };

export default function Page() {
  return <CenteredSignInView />;
}
