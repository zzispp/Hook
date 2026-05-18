import type { LangCode } from './locales-config';

import { cache } from 'react';
import { headers } from 'next/headers';
import { createInstance } from 'i18next';
import { initReactI18next } from 'react-i18next/initReactI18next';

import {
  defaultNS,
  i18nOptions,
  fallbackLng,
  i18nResourceLoader,
} from './locales-config';

// ----------------------------------------------------------------------

/**
 * Internationalization configuration for Next.js server-side.
 *
 * Language detection uses only the request Accept-Language header.
 */

const DEFAULT_ACCEPT_LANGUAGE_WEIGHT = 1;
const DISABLED_ACCEPT_LANGUAGE_WEIGHT = 0;
const Q_PARAM_PREFIX = 'q=';
const ENGLISH_LANGUAGE_PREFIX = 'en';

type HeaderLanguage = {
  readonly value: string;
  readonly weight: number;
  readonly index: number;
};

function isChineseLanguage(value?: string | null): boolean {
  if (!value) {
    return false;
  }

  const lower = value.toLowerCase();

  return lower === 'cn' || lower === 'zh' || lower.startsWith('zh-') || lower.startsWith('zh_');
}

function isEnglishLanguage(value?: string | null): boolean {
  if (!value) {
    return false;
  }

  const lower = value.toLowerCase();

  return lower === ENGLISH_LANGUAGE_PREFIX || lower.startsWith(`${ENGLISH_LANGUAGE_PREFIX}-`);
}

function parseHeaderLanguage(part: string, index: number): HeaderLanguage | undefined {
  const [rawValue, ...rawParams] = part.split(';');
  const value = rawValue?.trim();

  if (!value) {
    return undefined;
  }

  const qParam = rawParams.map((param) => param.trim()).find((param) => param.startsWith(Q_PARAM_PREFIX));
  const parsedWeight = qParam ? Number(qParam.slice(Q_PARAM_PREFIX.length)) : DEFAULT_ACCEPT_LANGUAGE_WEIGHT;
  const weight = Number.isFinite(parsedWeight) ? parsedWeight : DISABLED_ACCEPT_LANGUAGE_WEIGHT;

  return { value, weight, index };
}

function detectHeaderLanguage(header?: string | null): LangCode {
  const preferred = (header ?? '')
    .split(',')
    .map(parseHeaderLanguage)
    .filter((item): item is HeaderLanguage => Boolean(item))
    .sort((left, right) => right.weight - left.weight || left.index - right.index)[0];

  if (isChineseLanguage(preferred?.value)) {
    return 'cn';
  }

  return isEnglishLanguage(preferred?.value) ? 'en' : fallbackLng;
}

export async function detectLanguage() {
  const headerStore = await headers();

  return detectHeaderLanguage(headerStore.get('accept-language'));
}

// ----------------------------------------------------------------------

export async function initServerI18next(lang: LangCode, namespace: string) {
  const i18nInstance = createInstance();
  const initOptions = i18nOptions(lang, namespace);

  await i18nInstance.use(initReactI18next).use(i18nResourceLoader).init(initOptions);

  return i18nInstance;
}

// ----------------------------------------------------------------------

type Options = Record<string, unknown> & {
  keyPrefix?: string;
};

export const getServerTranslations = cache(async (namespace = defaultNS, options: Options = {}) => {
  const lang = await detectLanguage();
  const i18nextInstance = await initServerI18next(lang, namespace);

  return {
    t: i18nextInstance.getFixedT(
      lang,
      Array.isArray(namespace) ? namespace[0] : namespace,
      options?.keyPrefix
    ),
    i18n: i18nextInstance,
  };
});
