import type { Metadata } from 'next';
import type { ApiEnvelope } from 'src/types/rbac';
import type { I18nResourceResponse } from 'src/types/i18n';

import { cache } from 'react';
import { join } from 'node:path';
import { readFile } from 'node:fs/promises';

import { CONFIG } from 'src/global-config';
import { detectLanguage } from 'src/locales/server';
import enCommon from 'src/locales/langs/en/common.json';
import cnCommon from 'src/locales/langs/cn/common.json';
import { fallbackLng, type LangCode } from 'src/locales/locales-config';
import {
  navTranslationKey,
  type DashboardMenuCode,
} from 'src/layouts/dashboard/dashboard-menu-values';

const ADMIN_NAMESPACE = 'admin';
const ADMIN_RESOURCE_PATH = '/api/i18n/resources';
const STATIC_ADMIN_RESOURCE_DIR = '../hook_backend/src/migration/defaults/i18n';
const DASHBOARD_TITLE_KEY = 'nav.dashboard';

const STATIC_ADMIN_RESOURCE_FILES: Record<LangCode, string> = {
  cn: 'admin.cn.json',
  en: 'admin.en.json',
};

const STATIC_COMMON_RESOURCES: Record<LangCode, typeof cnCommon> = {
  cn: cnCommon,
  en: enCommon,
};

export async function dashboardPageMetadata(code: DashboardMenuCode): Promise<Metadata> {
  const lang = CONFIG.isStaticExport ? fallbackLng : await detectLanguage();
  const resources = await getAdminResources(lang);
  const title = resourceString(resources, navTranslationKey(code));
  const dashboardTitle = resourceString(STATIC_COMMON_RESOURCES[lang], DASHBOARD_TITLE_KEY);

  return {
    title: `${title} | ${dashboardTitle} - ${CONFIG.appName}`,
  };
}

const getAdminResources = cache(async (lang: LangCode) => {
  if (CONFIG.isStaticExport) {
    return staticAdminResources(lang);
  }

  const serverUrl = CONFIG.serverUrl.trim();
  if (!serverUrl) {
    throw new Error('NEXT_PUBLIC_SERVER_URL is required for dashboard metadata i18n.');
  }

  const url = new URL(ADMIN_RESOURCE_PATH, withTrailingSlash(serverUrl));
  url.searchParams.set('lang', lang);
  url.searchParams.set('namespace', ADMIN_NAMESPACE);

  const response = await fetch(url, { cache: 'no-store' });
  if (!response.ok) {
    throw new Error(`Failed to load admin i18n resources for metadata: ${response.status}`);
  }

  const payload = (await response.json()) as ApiEnvelope<I18nResourceResponse>;
  if (!payload.success || !payload.data) {
    throw new Error(payload.message || 'Failed to load admin i18n resources for metadata.');
  }

  return payload.data.resources;
});

async function staticAdminResources(lang: LangCode) {
  const content = await readFile(
    join(process.cwd(), STATIC_ADMIN_RESOURCE_DIR, STATIC_ADMIN_RESOURCE_FILES[lang]),
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
    throw new Error(`Missing dashboard metadata translation: ${path}`);
  }

  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function withTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}
