import type { Metadata } from 'next';

import { authPageMetadata } from 'src/app/auth/page-metadata';

import { JwtSignUpView } from 'src/auth/view/jwt';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return authPageMetadata('signUp.title');
}

export default function Page() {
  return <JwtSignUpView />;
}
