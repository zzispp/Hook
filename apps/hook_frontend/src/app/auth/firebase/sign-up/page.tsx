import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { FirebaseSignUpView } from 'src/auth/view/firebase';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign up | Firebase - ${CONFIG.appName}` };

export default function Page() {
  return <FirebaseSignUpView />;
}
