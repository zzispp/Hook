import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AmplifySignInView } from 'src/auth/view/amplify';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | Amplify - ${CONFIG.appName}` };

export default function Page() {
  return <AmplifySignInView />;
}
