'use client';

import type { NavSectionProps } from 'src/components/nav-section';

import { useMemo } from 'react';

import { paths } from 'src/routes/paths';
import { usePathname } from 'src/routes/hooks';

import { useNavbar } from 'src/actions/rbac';
import { DASHBOARD_MENU_TITLES } from './dashboard-menu-values';

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];

type DashboardBreadcrumbFallback = {
  heading: string;
  section?: string;
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

  return useMemo(() => {
    const current = currentNavItem(navbar.data, pathname);
    const heading = current?.item.title ?? fallback.heading;
    const links: DashboardBreadcrumbLink[] = [
      { name: rootTitle(navbar.data), href: paths.dashboard.root },
      ...(current?.section.subheader || fallback.section
        ? [{ name: current?.section.subheader ?? fallback.section ?? '' }]
        : []),
      { name: heading },
    ];

    return {
      heading,
      links,
      isLoading: navbar.isLoading,
      error: navbar.error,
    };
  }, [fallback.heading, fallback.section, navbar.data, navbar.error, navbar.isLoading, pathname]);
}

function rootTitle(data: NavData) {
  const rootItem = data.flatMap((section) => section.items).find((item) => samePath(item.path, paths.dashboard.root));
  return rootItem?.title ?? DASHBOARD_MENU_TITLES.dashboard;
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
