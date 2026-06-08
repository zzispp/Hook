import type { Metadata } from 'next';

import { authPageMetadata } from 'src/app/auth/page-metadata';

import { JwtSignInView } from 'src/auth/view/jwt';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return authPageMetadata('signIn.title');
}

export default function Page() {
  return <JwtSignInView />;
}
