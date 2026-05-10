'use client';

import type { NavSectionProps } from 'src/components/nav-section';

import { useMemo } from 'react';

import { paths } from 'src/routes/paths';
import { usePathname } from 'src/routes/hooks';

import { useNavbar } from 'src/actions/rbac';
import { useTranslate } from 'src/locales/use-locales';

import {
  navTranslationKey,
  DASHBOARD_MENU_CODES,
  type DashboardMenuCode,
  type DashboardSectionCode,
} from './dashboard-menu-values';

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];

type DashboardBreadcrumbFallback = {
  headingCode: DashboardMenuCode;
  sectionCode?: DashboardSectionCode;
};

type DashboardBreadcrumbLink = {
  name: string;
  href?: string;
};

const PATH_SUFFIX_PATTERN = /[?#]/;
const TRAILING_SLASH_PATTERN = /\/+$/;

export function useDashboardBreadcrumbs(fallback: DashboardBreadcrumbFallback) {
  const pathname = usePathname();
  const navbar = useNavbar();
  const { t } = useTranslate('admin');

  return useMemo(() => {
    const current = currentNavItem(navbar.data, pathname);
    const heading = current?.item.title ?? String(t(navTranslationKey(fallback.headingCode)));
    const section = fallback.sectionCode ? String(t(navTranslationKey(fallback.sectionCode))) : undefined;
    const links: DashboardBreadcrumbLink[] = [
      { name: rootTitle(navbar.data, t), href: paths.dashboard.root },
      ...(current?.section.subheader || section ? [{ name: current?.section.subheader ?? section ?? '' }] : []),
      { name: heading },
    ];

    return {
      heading,
      links,
      isLoading: navbar.isLoading,
      error: navbar.error,
    };
  }, [
    fallback.headingCode,
    fallback.sectionCode,
    navbar.data,
    navbar.error,
    navbar.isLoading,
    pathname,
    t,
  ]);
}

function rootTitle(data: NavData, t: ReturnType<typeof useTranslate>['t']) {
  const rootItem = data.flatMap((section) => section.items).find((item) => samePath(item.path, paths.dashboard.root));
  return rootItem?.title ?? String(t(navTranslationKey(DASHBOARD_MENU_CODES.dashboard)));
}

function currentNavItem(data: NavData, pathname: string) {
  const route = normalizePath(pathname);

  for (const section of data) {
    for (const item of section.items) {
      const match = matchingItem(item, route);
      if (match) {
        return { section, item: match };
      }
    }
  }

  return null;
}

function matchingItem(item: NavItem, route: string): NavItem | null {
  const itemPath = normalizePath(item.path);

  if (samePath(itemPath, route) || (item.deepMatch && route.startsWith(`${itemPath}/`))) {
    return item;
  }

  for (const child of item.children ?? []) {
    const match = matchingItem(child, route);
    if (match) {
      return match;
    }
  }

  return null;
}

function samePath(left: string, right: string) {
  return normalizePath(left) === normalizePath(right);
}

function normalizePath(path: string) {
  const [pathname] = path.split(PATH_SUFFIX_PATTERN);
  const normalized = pathname.replace(TRAILING_SLASH_PATTERN, '');
  return normalized || '/';
}
