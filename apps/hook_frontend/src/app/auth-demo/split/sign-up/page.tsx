import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SplitSignUpView } from 'src/auth/view/auth-demo/split';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign up | Layout split - ${CONFIG.appName}` };

export default function Page() {
  return <SplitSignUpView />;
}
