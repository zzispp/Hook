import type { PublicSiteInfo } from 'src/types/system-setting';

export function normalizePublicSiteInfo(siteInfo?: PublicSiteInfo): PublicSiteInfo | undefined {
  if (!siteInfo) {
    return undefined;
  }

  const siteName = siteInfo.site_name.trim();

  if (!siteName) {
    return undefined;
  }

  return {
    ...siteInfo,
    site_name: siteName,
    site_subtitle: siteInfo.site_subtitle.trim(),
  };
}
