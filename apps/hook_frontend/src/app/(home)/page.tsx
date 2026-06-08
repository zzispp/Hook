import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import enLanding from 'src/locales/langs/en/landing.json';
import cnLanding from 'src/locales/langs/cn/landing.json';
import { getServerTranslations } from 'src/locales/server';
import LandingPage from 'src/react-bits/pages/LandingPage';
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

  return {
    title: t('pageMetadata.title'),
    description: t('pageMetadata.description'),
  };
}

export default function Page() {
  return <LandingPage />;
}
