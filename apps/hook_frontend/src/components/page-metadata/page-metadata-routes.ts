import { paths } from 'src/routes/paths';

import {
  DASHBOARD_MENU_CODES,
  type DashboardMenuCode,
} from 'src/layouts/dashboard/dashboard-menu-values';

type DashboardTitleRoute = {
  readonly path: string;
  readonly code: DashboardMenuCode;
  readonly deep?: boolean;
};

const PATH_SUFFIX_PATTERN = /[?#]/;
const TRAILING_SLASH_PATTERN = /\/+$/;
const AFFILIATE_PATH = '/dashboard/affiliate';
const ADMIN_AFFILIATES_PATH = '/dashboard/admin/affiliates';
const ADMIN_SCHEDULED_TASKS_PATH = '/dashboard/admin/scheduled-tasks';

const DASHBOARD_TITLE_ROUTES: readonly DashboardTitleRoute[] = [
  { path: paths.dashboard.profileChangePassword, code: DASHBOARD_MENU_CODES.profile },
  { path: paths.dashboard.profileVerifyEmail, code: DASHBOARD_MENU_CODES.profile },
  { path: paths.dashboard.announcementDetail, code: DASHBOARD_MENU_CODES.announcements },
  { path: paths.dashboard.admin.cacheMonitoring, code: DASHBOARD_MENU_CODES.cacheMonitoring },
  { path: paths.dashboard.admin.costAnalysis, code: DASHBOARD_MENU_CODES.costAnalysis },
  { path: ADMIN_SCHEDULED_TASKS_PATH, code: DASHBOARD_MENU_CODES.scheduledTaskManagement },
  { path: paths.dashboard.admin.cardCodes, code: DASHBOARD_MENU_CODES.cardCodeManagement },
  { path: paths.dashboard.admin.modelStatusChecks, code: DASHBOARD_MENU_CODES.modelStatusChecks },
  {
    path: paths.dashboard.admin.performanceMonitoring,
    code: DASHBOARD_MENU_CODES.performanceMonitoring,
  },
  { path: paths.dashboard.admin.routing, code: DASHBOARD_MENU_CODES.routing },
  { path: paths.dashboard.admin.requestRecords, code: DASHBOARD_MENU_CODES.requestRecords },
  { path: paths.dashboard.admin.userGroups, code: DASHBOARD_MENU_CODES.userGroups },
  { path: paths.dashboard.admin.userStats, code: DASHBOARD_MENU_CODES.userStats },
  { path: ADMIN_AFFILIATES_PATH, code: DASHBOARD_MENU_CODES.affiliateManagement },
  { path: paths.dashboard.admin.announcements, code: DASHBOARD_MENU_CODES.announcementManagement },
  { path: paths.dashboard.admin.translations, code: DASHBOARD_MENU_CODES.translationManagement },
  { path: paths.dashboard.admin.providers, code: DASHBOARD_MENU_CODES.providerManagement },
  { path: paths.dashboard.admin.recharges, code: DASHBOARD_MENU_CODES.rechargeManagement },
  { path: paths.dashboard.admin.settings, code: DASHBOARD_MENU_CODES.systemSettings },
  { path: paths.dashboard.admin.tickets, code: DASHBOARD_MENU_CODES.ticketManagement },
  { path: paths.dashboard.admin.wallets, code: DASHBOARD_MENU_CODES.walletManagement },
  { path: paths.dashboard.admin.groups, code: DASHBOARD_MENU_CODES.billingGroups },
  { path: paths.dashboard.admin.models, code: DASHBOARD_MENU_CODES.modelManagement },
  { path: paths.dashboard.admin.tokens, code: DASHBOARD_MENU_CODES.tokenManagement },
  { path: paths.dashboard.admin.roles, code: DASHBOARD_MENU_CODES.roleManagement },
  { path: paths.dashboard.admin.users, code: DASHBOARD_MENU_CODES.userManagement },
  { path: paths.dashboard.admin.menus, code: DASHBOARD_MENU_CODES.menuManagement },
  { path: paths.dashboard.admin.apis, code: DASHBOARD_MENU_CODES.apiManagement },
  { path: paths.dashboard.usageRecords, code: DASHBOARD_MENU_CODES.usageRecords },
  { path: paths.dashboard.modelStatus, code: DASHBOARD_MENU_CODES.modelStatus },
  { path: paths.dashboard.announcements, code: DASHBOARD_MENU_CODES.announcements, deep: true },
  { path: AFFILIATE_PATH, code: DASHBOARD_MENU_CODES.affiliateCenter },
  { path: paths.dashboard.tickets, code: DASHBOARD_MENU_CODES.tickets },
  { path: paths.dashboard.groups, code: DASHBOARD_MENU_CODES.billingGroupCatalog },
  { path: paths.dashboard.models, code: DASHBOARD_MENU_CODES.modelCatalog },
  { path: paths.dashboard.tokens, code: DASHBOARD_MENU_CODES.apiTokens },
  { path: paths.dashboard.wallet, code: DASHBOARD_MENU_CODES.walletCenter },
  { path: paths.dashboard.profile, code: DASHBOARD_MENU_CODES.profile, deep: true },
  { path: paths.dashboard.root, code: DASHBOARD_MENU_CODES.dashboard },
];

const AUTH_TITLE_KEYS = {
  [paths.auth.jwt.signIn]: 'signIn.title',
  [paths.auth.jwt.signUp]: 'signUp.title',
  [paths.auth.jwt.forgotPassword]: 'forgotPassword.title',
  [paths.auth.jwt.resetPassword]: 'resetPassword.title',
  '/auth/oauth/callback': 'social.oauthProcessing',
} as const;

const STATIC_PAGE_TITLE_KEYS = {
  [paths.about]: 'aboutUs',
  [paths.comingSoon]: 'comingSoon',
  [paths.maintenance]: 'maintenance',
  [paths.payment]: 'payment',
  [paths.pricing]: 'pricing',
  [paths.page403]: 'page403',
  [paths.page404]: 'page404',
  [paths.page500]: 'page500',
} as const;

export function findDashboardMenuCode(pathname: string): DashboardMenuCode | undefined {
  const route = normalizePath(pathname);
  return DASHBOARD_TITLE_ROUTES.find((item) => matchesRoute(item, route))?.code;
}

export function findAuthTitleKey(pathname: string) {
  const route = normalizePath(pathname);
  const exactKey = AUTH_TITLE_KEYS[route as keyof typeof AUTH_TITLE_KEYS];

  if (exactKey) {
    return exactKey;
  }

  return route.startsWith('/auth/oauth/callback/')
    ? AUTH_TITLE_KEYS['/auth/oauth/callback']
    : undefined;
}

export function findStaticPageTitleKey(pathname: string) {
  const route = normalizePath(pathname);
  return STATIC_PAGE_TITLE_KEYS[route as keyof typeof STATIC_PAGE_TITLE_KEYS];
}

export function isHomePath(pathname: string) {
  return normalizePath(pathname) === '/';
}

function matchesRoute(item: DashboardTitleRoute, route: string) {
  const itemPath = normalizePath(item.path);
  return item.deep ? route === itemPath || route.startsWith(`${itemPath}/`) : route === itemPath;
}

function normalizePath(path: string) {
  const [pathname] = path.split(PATH_SUFFIX_PATTERN);
  const normalized = pathname.replace(TRAILING_SLASH_PATTERN, '');
  return normalized || '/';
}
