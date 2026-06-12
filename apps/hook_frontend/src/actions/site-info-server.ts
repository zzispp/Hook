import type { ApiEnvelope } from 'src/types/rbac';
import type { PublicSiteInfo } from 'src/types/system-setting';

import { cache } from 'react';

import { endpoints } from 'src/lib/axios';
import { CONFIG } from 'src/global-config';

export const getSiteInfo = cache(async (): Promise<PublicSiteInfo> => {
  if (CONFIG.isStaticExport) {
    throw new Error('Site info is unavailable during static export.');
  }

  const serverUrl = CONFIG.serverUrl.trim();
  if (!serverUrl) {
    throw new Error('NEXT_PUBLIC_SERVER_URL is required for site info.');
  }

  const response = await fetch(new URL(endpoints.siteInfo, withTrailingSlash(serverUrl)), {
    cache: 'no-store',
  });
  if (!response.ok) {
    throw new Error(`Failed to load site info: ${response.status}`);
  }

  return requireSiteInfo((await response.json()) as ApiEnvelope<PublicSiteInfo>);
});

function requireSiteInfo(payload: ApiEnvelope<PublicSiteInfo>) {
  if (!payload.success || !payload.data) {
    throw new Error(payload.message || 'Failed to load site info.');
  }

  return payload.data;
}

function withTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}
