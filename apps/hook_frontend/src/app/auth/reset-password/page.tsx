import type { Metadata } from 'next';

import { authPageMetadata } from 'src/app/auth/page-metadata';

import { JwtResetPasswordView } from 'src/auth/view/jwt';

export function generateMetadata(): Promise<Metadata> {
  return authPageMetadata('resetPassword.title');
}

export default function Page() {
  return <JwtResetPasswordView />;
}
