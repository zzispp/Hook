'use client';

import type { InitOptions } from 'i18next';
import type { LangCode } from './locales-config';

import i18next from 'i18next';
import { useEffect } from 'react';
import { getStorage } from 'minimal-shared/utils';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import {
  i18nOptions,
  fallbackLng,
  storageConfig,
  i18nResourceLoader,
} from './locales-config';

// ----------------------------------------------------------------------

const i18nextLng = getStorage(
  storageConfig.localStorage.key,
  storageConfig.localStorage.autoDetection ? undefined : fallbackLng
) as LangCode | null;

/**
 * Initialize i18next
 */
const initOptions: InitOptions = {
  ...i18nOptions(i18nextLng ?? fallbackLng),
  detection: {
    caches: ['localStorage'],
    lookupLocalStorage: storageConfig.localStorage.key,
    convertDetectedLanguage: normalizeDetectedLanguage,
  },
};

i18next.use(LanguageDetector).use(initReactI18next).use(i18nResourceLoader).init(initOptions);

// ----------------------------------------------------------------------

type I18nProviderProps = {
  lang?: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  useEffect(() => {
    const storedLang = getStorage(storageConfig.localStorage.key) as LangCode | null;

    if (!storedLang && lang && i18next.language !== lang) {
      i18next.changeLanguage(lang);
    }
  }, [lang]);

  return <Provider i18n={i18next}>{children}</Provider>;
}

function normalizeDetectedLanguage(lang: string) {
  const lower = lang.toLowerCase();

  if (lower === 'cn' || lower.startsWith('zh')) {
    return 'cn';
  }

  if (lower === 'en' || lower.startsWith('en')) {
    return 'en';
  }

  return fallbackLng;
}
