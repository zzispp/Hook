'use client';

import type { TFunction } from 'i18next';
import type { DashboardMenuCode } from 'src/layouts/dashboard/dashboard-menu-values';

import { useState, useEffect } from 'react';

import { usePathname } from 'src/routes/hooks';

import { useI18nResource } from 'src/actions/i18n';
import { useTranslate } from 'src/locales/use-locales';
import { useSiteInfo } from 'src/actions/system-settings';
import { navTranslationKey } from 'src/layouts/dashboard/dashboard-menu-values';

import {
  isHomePath,
  findAuthTitleKey,
  findDashboardMenuCode,
  findStaticPageTitleKey,
} from './page-metadata-routes';

type PageMetadata = {
  readonly title: string;
  readonly description?: string;
};

type SiteMetadata = {
  readonly siteName: string;
  readonly siteSubtitle?: string;
};

export function PageMetadataSync() {
  const pathname = usePathname();
  const site = useSiteInfo();

  if (site.error) {
    throw site.error;
  }

  if (!site.data) {
    return null;
  }

  const siteName = nonEmptyString(site.data.site_name);

  if (!siteName) {
    throw new Error('Site name is required for page metadata.');
  }

  const siteMetadata = { siteName, siteSubtitle: nonEmptyString(site.data.site_subtitle) };
  const dashboardCode = findDashboardMenuCode(pathname);
  const authTitleKey = findAuthTitleKey(pathname);
  const staticTitleKey = findStaticPageTitleKey(pathname);

  if (isHomePath(pathname)) {
    return <HomeMetadataSync {...siteMetadata} />;
  }

  if (dashboardCode) {
    return <DashboardMetadataSync code={dashboardCode} {...siteMetadata} />;
  }

  if (authTitleKey) {
    return <AuthMetadataSync titleKey={authTitleKey} {...siteMetadata} />;
  }

  if (staticTitleKey) {
    return <StaticPageMetadataSync titleKey={staticTitleKey} {...siteMetadata} />;
  }

  return null;
}

function HomeMetadataSync({ siteName, siteSubtitle }: SiteMetadata) {
  const { t } = useTranslate('landing');
  const titleSuffix = requiredTranslation(t, 'pageMetadata.titleSuffix');
  const description = siteSubtitle ?? requiredTranslation(t, 'pageMetadata.description');

  return <DocumentMetadataSync metadata={{ title: `${siteName} | ${titleSuffix}`, description }} />;
}

function DashboardMetadataSync({
  code,
  siteName,
  siteSubtitle,
}: SiteMetadata & { code: DashboardMenuCode }) {
  const loaded = useNamespaceResourceSync('admin');
  const { t: adminT } = useTranslate('admin');
  const { t: commonT } = useTranslate('common');

  if (!loaded) {
    return null;
  }

  const pageTitle = requiredTranslation(adminT, navTranslationKey(code));
  const dashboardTitle = requiredTranslation(commonT, 'nav.dashboard');

  return (
    <DocumentMetadataSync
      metadata={{
        title: `${pageTitle} | ${dashboardTitle} - ${siteName}`,
        description: siteSubtitle,
      }}
    />
  );
}

function AuthMetadataSync({
  titleKey,
  siteName,
  siteSubtitle,
}: SiteMetadata & { titleKey: string }) {
  const loaded = useNamespaceResourceSync('auth');
  const { t } = useTranslate('auth');

  if (!loaded) {
    return null;
  }

  const pageTitle = requiredTranslation(t, titleKey);

  return (
    <DocumentMetadataSync
      metadata={{ title: `${pageTitle} | ${siteName}`, description: siteSubtitle }}
    />
  );
}

function StaticPageMetadataSync({
  titleKey,
  siteName,
  siteSubtitle,
}: SiteMetadata & { titleKey: string }) {
  const { t } = useTranslate('common');
  const pageTitle = requiredTranslation(t, `metadata.pages.${titleKey}`);

  return (
    <DocumentMetadataSync
      metadata={{ title: `${pageTitle} - ${siteName}`, description: siteSubtitle }}
    />
  );
}

function DocumentMetadataSync({ metadata }: { readonly metadata: PageMetadata }) {
  useEffect(() => {
    document.title = metadata.title;
    syncMetaTag('name', 'description', metadata.description);
    syncMetaTag('property', 'og:title', metadata.title);
    syncMetaTag('name', 'twitter:title', metadata.title);
    syncMetaTag('property', 'og:description', metadata.description);
    syncMetaTag('name', 'twitter:description', metadata.description);
  }, [metadata.description, metadata.title]);

  return null;
}

function useNamespaceResourceSync(namespace: string) {
  const { i18n, currentLang } = useTranslate();
  const resource = useI18nResource(currentLang.value, namespace);
  const [loadedKey, setLoadedKey] = useState('');

  useEffect(() => {
    if (!resource.data) {
      return;
    }

    i18n.addResourceBundle(
      resource.data.lang,
      resource.data.namespace,
      resource.data.resources,
      true,
      true
    );
    setLoadedKey(resourceKey(resource.data.lang, resource.data.namespace));
  }, [i18n, resource.data]);

  if (resource.error) {
    throw resource.error;
  }

  return loadedKey === resourceKey(currentLang.value, namespace);
}

function syncMetaTag(attributeName: 'name' | 'property', attributeValue: string, content?: string) {
  const selector = `meta[${attributeName}="${attributeValue}"]`;
  let element = document.head.querySelector<HTMLMetaElement>(selector);
  const nextContent = nonEmptyString(content);

  if (!nextContent) {
    element?.remove();
    return;
  }

  if (!element) {
    element = document.createElement('meta');
    element.setAttribute(attributeName, attributeValue);
    document.head.appendChild(element);
  }

  element.setAttribute('content', nextContent);
}

function requiredTranslation(t: TFunction, key: string) {
  const value = t(key);
  const text = typeof value === 'string' ? nonEmptyString(value) : undefined;

  if (!text || text === key) {
    throw new Error(`Missing page metadata translation: ${key}`);
  }

  return text;
}

function nonEmptyString(value?: string | null) {
  const text = value?.trim();
  return text || undefined;
}

function resourceKey(lang: string, namespace: string) {
  return `${lang}:${namespace}`;
}
