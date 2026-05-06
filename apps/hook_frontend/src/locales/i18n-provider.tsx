'use client';

import type { InitOptions } from 'i18next';
import type { LangCode } from './locales-config';

import i18next from 'i18next';
import { useEffect } from 'react';
import { getStorage } from 'minimal-shared/utils';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import { CONFIG } from 'src/global-config';

import { i18nOptions, fallbackLng, storageConfig, i18nResourceLoader } from './locales-config';

// ----------------------------------------------------------------------

let i18nextLng;

if (CONFIG.isStaticExport) {
  i18nextLng = getStorage(
    storageConfig.localStorage.key,
    storageConfig.localStorage.autoDetection ? undefined : fallbackLng
  ) as LangCode;
}

/**
 * Initialize i18next
 */
const initOptions: InitOptions = CONFIG.isStaticExport
  ? { ...i18nOptions(i18nextLng), detection: { caches: ['localStorage'] } }
  : { ...i18nOptions(), detection: { caches: ['cookie'] } };

i18next.use(LanguageDetector).use(initReactI18next).use(i18nResourceLoader).init(initOptions);

// ----------------------------------------------------------------------

type I18nProviderProps = {
  lang?: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  /**
   * Cookie storage
   * Restore the selected language after a page refresh.
   * since i18next might lose the language state on reload.
   */
  useEffect(() => {
    if (lang && i18next.language !== lang) {
      i18next.changeLanguage(lang);
    }
  }, [lang]);

  return <Provider i18n={i18next}>{children}</Provider>;
}
