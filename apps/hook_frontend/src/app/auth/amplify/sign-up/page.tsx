import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AmplifySignUpView } from 'src/auth/view/amplify';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign up | Amplify - ${CONFIG.appName}` };

export default function Page() {
  return <AmplifySignUpView />;
}
