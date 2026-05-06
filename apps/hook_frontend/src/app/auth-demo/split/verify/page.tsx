import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SplitVerifyView } from 'src/auth/view/auth-demo/split';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Verify | Layout split - ${CONFIG.appName}` };

export default function Page() {
  return <SplitVerifyView />;
}
