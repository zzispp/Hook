'use client';

import type { TokenScope } from './api-token-management-types';

import { getApiTokenSecret, getAdminApiTokenSecret } from 'src/actions/api-tokens';

import { fetchTokenModelIds } from './api-token-cc-switch-utils';

type LoadTokenContextArgs = {
  scope: TokenScope;
  tokenId: string;
  baseUrl: string;
};

export async function loadCcSwitchTokenContext({
  scope,
  tokenId,
  baseUrl,
}: LoadTokenContextArgs) {
  const rawToken = await loadRawToken(scope, tokenId);
  const modelIds = await fetchTokenModelIds(rawToken, baseUrl);
  return { rawToken, modelIds };
}

async function loadRawToken(scope: TokenScope, tokenId: string) {
  const response =
    scope === 'admin' ? await getAdminApiTokenSecret(tokenId) : await getApiTokenSecret(tokenId);
  return response.raw_token.trim();
}
