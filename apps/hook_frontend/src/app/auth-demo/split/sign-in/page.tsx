import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SplitSignInView } from 'src/auth/view/auth-demo/split';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | Layout split - ${CONFIG.appName}` };

export default function Page() {
  return <SplitSignInView />;
}
