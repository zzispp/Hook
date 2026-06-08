import type { Metadata } from 'next';

import { authPageMetadata } from 'src/app/auth/page-metadata';

import { JwtForgotPasswordView } from 'src/auth/view/jwt';

export function generateMetadata(): Promise<Metadata> {
  return authPageMetadata('forgotPassword.title');
}

export default function Page() {
  return <JwtForgotPasswordView />;
}
