import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { JwtResetPasswordView } from 'src/auth/view/jwt';

export const metadata: Metadata = { title: `Reset password | ${CONFIG.appName}` };

export default function Page() {
  return <JwtResetPasswordView />;
}
