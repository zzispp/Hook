import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import LandingPage from 'src/home/pages/LandingPage';
import enLanding from 'src/locales/langs/en/landing.json';
import cnLanding from 'src/locales/langs/cn/landing.json';
import { getServerTranslations } from 'src/locales/server';
import { getSiteInfo } from 'src/actions/site-info-server';
import { fallbackLng, type LangCode } from 'src/locales/locales-config';

// ----------------------------------------------------------------------

const STATIC_LANDING_METADATA: Record<LangCode, typeof cnLanding.pageMetadata> = {
  cn: cnLanding.pageMetadata,
  en: enLanding.pageMetadata,
};

export async function generateMetadata(): Promise<Metadata> {
  if (CONFIG.isStaticExport) {
    const pageMetadata = STATIC_LANDING_METADATA[fallbackLng];

    return {
      title: pageMetadata.title,
      description: pageMetadata.description,
    };
  }

  const { t } = await getServerTranslations('landing');
  const site = await loadMetadataSiteInfo();

  if (!site) {
    return {
      title: t('siteInfoStatus.errorTitle'),
      description: t('pageMetadata.description'),
    };
  }

  return {
    title: `${site.site_name.trim()} | ${t('pageMetadata.titleSuffix')}`,
    description: site.site_subtitle.trim() || t('pageMetadata.description'),
  };
}

async function loadMetadataSiteInfo() {
  try {
    const site = await getSiteInfo();
    const siteName = site.site_name.trim();

    return siteName ? site : null;
  } catch {
    return null;
  }
}

export default function Page() {
  return <LandingPage />;
}
