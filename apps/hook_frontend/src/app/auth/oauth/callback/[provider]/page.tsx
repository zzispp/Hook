import type { Metadata } from 'next';
import type { IdentityProvider } from 'src/types/rbac';

import { CONFIG } from 'src/global-config';

import { JwtOAuthCallbackView } from 'src/auth/view/jwt';

export const metadata: Metadata = { title: `OAuth callback | ${CONFIG.appName}` };

type Props = {
  params: Promise<{ provider: string }>;
};

export default async function Page({ params }: Props) {
  const { provider } = await params;

  return <JwtOAuthCallbackView provider={oauthProvider(provider)} />;
}

function oauthProvider(value: string): Extract<IdentityProvider, 'github' | 'google'> {
  if (value === 'github' || value === 'google') {
    return value;
  }
  return 'github';
}
