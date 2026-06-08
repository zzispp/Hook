import type { Metadata } from 'next';
import type { ApiEnvelope } from 'src/types/rbac';
import type { I18nResourceResponse } from 'src/types/i18n';

import { cache } from 'react';
import { join } from 'node:path';
import { readFile } from 'node:fs/promises';

import { CONFIG } from 'src/global-config';
import { detectLanguage } from 'src/locales/server';
import { fallbackLng, type LangCode } from 'src/locales/locales-config';

const AUTH_NAMESPACE = 'auth';
const AUTH_RESOURCE_PATH = '/api/i18n/resources';
const STATIC_AUTH_RESOURCE_DIR = '../hook_backend/src/migration/defaults/i18n';

const STATIC_AUTH_RESOURCE_FILES: Record<LangCode, string> = {
  cn: 'auth.cn.json',
  en: 'auth.en.json',
};

export async function authPageMetadata(titlePath: string): Promise<Metadata> {
  const lang = CONFIG.isStaticExport ? fallbackLng : await detectLanguage();
  const resources = await getAuthResources(lang);
  const title = resourceString(resources, titlePath);

  return {
    title: `${title} | ${CONFIG.appName}`,
  };
}

const getAuthResources = cache(async (lang: LangCode) => {
  if (CONFIG.isStaticExport) {
    return staticAuthResources(lang);
  }

  const serverUrl = CONFIG.serverUrl.trim();
  if (!serverUrl) {
    throw new Error('NEXT_PUBLIC_SERVER_URL is required for auth metadata i18n.');
  }

  const url = new URL(AUTH_RESOURCE_PATH, withTrailingSlash(serverUrl));
  url.searchParams.set('lang', lang);
  url.searchParams.set('namespace', AUTH_NAMESPACE);

  const response = await fetch(url, { cache: 'no-store' });
  if (!response.ok) {
    throw new Error(`Failed to load auth i18n resources for metadata: ${response.status}`);
  }

  const payload = (await response.json()) as ApiEnvelope<I18nResourceResponse>;
  if (!payload.success || !payload.data) {
    throw new Error(payload.message || 'Failed to load auth i18n resources for metadata.');
  }

  return payload.data.resources;
});

async function staticAuthResources(lang: LangCode) {
  const content = await readFile(
    join(process.cwd(), STATIC_AUTH_RESOURCE_DIR, STATIC_AUTH_RESOURCE_FILES[lang]),
    'utf8'
  );

  return JSON.parse(content) as Record<string, unknown>;
}

function resourceString(resources: Record<string, unknown>, path: string) {
  const value = path
    .split('.')
    .reduce<unknown>(
      (current, segment) => (isRecord(current) ? current[segment] : undefined),
      resources
    );

  if (typeof value !== 'string' || !value.trim()) {
    throw new Error(`Missing auth metadata translation: ${path}`);
  }

  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function withTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}
