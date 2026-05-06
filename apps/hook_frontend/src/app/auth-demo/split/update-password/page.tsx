import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SplitUpdatePasswordView } from 'src/auth/view/auth-demo/split';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Update password | Layout split - ${CONFIG.appName}` };

export default function Page() {
  return <SplitUpdatePasswordView />;
}
