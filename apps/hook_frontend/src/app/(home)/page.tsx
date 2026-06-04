import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import enLanding from 'src/locales/langs/en/landing.json';
import { getServerTranslations } from 'src/locales/server';
import LandingPage from 'src/react-bits/pages/LandingPage';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  if (CONFIG.isStaticExport) {
    return {
      title: enLanding.pageMetadata.title,
      description: enLanding.pageMetadata.description,
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
