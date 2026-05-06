import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CenteredSignUpView } from 'src/auth/view/auth-demo/centered';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign up | Layout centered - ${CONFIG.appName}` };

export default function Page() {
  return <CenteredSignUpView />;
}
