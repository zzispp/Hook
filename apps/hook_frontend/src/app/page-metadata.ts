import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { detectLanguage } from 'src/locales/server';
import enCommon from 'src/locales/langs/en/common.json';
import cnCommon from 'src/locales/langs/cn/common.json';
import { fallbackLng, type LangCode } from 'src/locales/locales-config';

type CommonPageTitleKey = keyof (typeof cnCommon)['metadata']['pages'];

const STATIC_COMMON_RESOURCES: Record<LangCode, typeof cnCommon> = {
  cn: cnCommon,
  en: enCommon,
};

export async function commonPageMetadata(key: CommonPageTitleKey): Promise<Metadata> {
  const lang = CONFIG.isStaticExport ? fallbackLng : await detectLanguage();
  const title = STATIC_COMMON_RESOURCES[lang].metadata.pages[key];

  if (!title.trim()) {
    throw new Error(`Missing common page metadata title: ${key}`);
  }

  return {
    title: `${title} - ${CONFIG.appName}`,
  };
}
