import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { JwtForgotPasswordView } from 'src/auth/view/jwt';

export const metadata: Metadata = { title: `Forgot password | ${CONFIG.appName}` };

export default function Page() {
  return <JwtForgotPasswordView />;
}
