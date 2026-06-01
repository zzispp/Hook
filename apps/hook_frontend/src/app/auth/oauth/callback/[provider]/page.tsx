import type { Metadata } from 'next';
import type { IdentityProvider } from 'src/types/rbac';

import { CONFIG } from 'src/global-config';

import { JwtOAuthCallbackView } from 'src/auth/view/jwt';

export const metadata: Metadata = { title: `OAuth callback | ${CONFIG.appName}` };

type OAuthProvider = Extract<IdentityProvider, 'github' | 'google'>;

type Props = {
  params: Promise<{ provider: string }>;
};

const OAUTH_PROVIDERS: readonly OAuthProvider[] = ['github', 'google'];

export function generateStaticParams() {
  return OAUTH_PROVIDERS.map((provider) => ({ provider }));
}

export default async function Page({ params }: Props) {
  const { provider } = await params;

  return <JwtOAuthCallbackView provider={oauthProvider(provider)} />;
}

function oauthProvider(value: string): OAuthProvider {
  if (value === 'github' || value === 'google') {
    return value;
  }
  return 'github';
}
