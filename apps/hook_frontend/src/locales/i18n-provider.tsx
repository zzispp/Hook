'use client';

import type { LangCode } from './locales-config';

import { createInstance } from 'i18next';
import { useRef, useEffect } from 'react';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import enCommon from './langs/en/common.json';
import cnCommon from './langs/cn/common.json';
import enNavbar from './langs/en/navbar.json';
import cnNavbar from './langs/cn/navbar.json';
import enMessages from './langs/en/messages.json';
import cnMessages from './langs/cn/messages.json';
import { i18nOptions, fallbackLng } from './locales-config';

// ----------------------------------------------------------------------

/**
 * Initialize i18next
 */
const I18N_NAMESPACES = ['common', 'messages', 'admin', 'auth', 'navbar'];

const I18N_RESOURCES = {
  cn: { common: cnCommon, messages: cnMessages, navbar: cnNavbar },
  en: { common: enCommon, messages: enMessages, navbar: enNavbar },
};

function createI18n(lang: LangCode) {
  const instance = createInstance();

  instance.use(initReactI18next).init({
    ...i18nOptions(lang),
    ns: I18N_NAMESPACES,
    resources: I18N_RESOURCES,
    initAsync: false,
  });

  return instance;
}

// ----------------------------------------------------------------------

type I18nProviderProps = {
  lang?: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  const mounted = useRef(false);
  const initialLang = lang ?? fallbackLng;
  const i18nRef = useRef<ReturnType<typeof createI18n> | null>(null);

  if (!i18nRef.current) {
    i18nRef.current = createI18n(initialLang);
  }

  const i18n = i18nRef.current;

  if (!mounted.current && i18n.language !== initialLang) {
    i18n.changeLanguage(initialLang);
  }

  useEffect(() => {
    mounted.current = true;
    const nextLang = lang ?? fallbackLng;

    if (i18n.language !== nextLang) {
      i18n.changeLanguage(nextLang);
    }
  }, [i18n, lang]);

  return <Provider i18n={i18n}>{children}</Provider>;
}
