const ROOTS = {
  AUTH: '/auth',
  DASHBOARD: '/dashboard',
};

// ----------------------------------------------------------------------

export const paths = {
  comingSoon: '/coming-soon',
  maintenance: '/maintenance',
  pricing: '/pricing',
  payment: '/payment',
  about: '/about-us',
  page403: '/error/403',
  page404: '/error/404',
  page500: '/error/500',
  // AUTH
  auth: {
    jwt: {
      signIn: `${ROOTS.AUTH}/sign-in`,
      signUp: `${ROOTS.AUTH}/sign-up`,
      forgotPassword: `${ROOTS.AUTH}/forgot-password`,
      resetPassword: `${ROOTS.AUTH}/reset-password`,
    },
  },
  // DASHBOARD
  dashboard: {
    root: ROOTS.DASHBOARD,
    models: `${ROOTS.DASHBOARD}/models`,
    groups: `${ROOTS.DASHBOARD}/groups`,
    announcements: `${ROOTS.DASHBOARD}/announcements`,
    announcementDetail: `${ROOTS.DASHBOARD}/announcements/detail`,
    tokens: `${ROOTS.DASHBOARD}/tokens`,
    usageRecords: `${ROOTS.DASHBOARD}/usage-records`,
    tickets: `${ROOTS.DASHBOARD}/tickets`,
    wallet: `${ROOTS.DASHBOARD}/wallet`,
    admin: {
      root: `${ROOTS.DASHBOARD}/admin`,
      users: `${ROOTS.DASHBOARD}/admin/users`,
      roles: `${ROOTS.DASHBOARD}/admin/roles`,
      apis: `${ROOTS.DASHBOARD}/admin/apis`,
      announcements: `${ROOTS.DASHBOARD}/admin/announcements`,
      menus: `${ROOTS.DASHBOARD}/admin/menus`,
      models: `${ROOTS.DASHBOARD}/admin/models`,
      providers: `${ROOTS.DASHBOARD}/admin/providers`,
      cacheMonitoring: `${ROOTS.DASHBOARD}/admin/cache-monitoring`,
      performanceMonitoring: `${ROOTS.DASHBOARD}/admin/performance-monitoring`,
      requestRecords: `${ROOTS.DASHBOARD}/admin/request-records`,
      tickets: `${ROOTS.DASHBOARD}/admin/tickets`,
      tokens: `${ROOTS.DASHBOARD}/admin/tokens`,
      groups: `${ROOTS.DASHBOARD}/admin/groups`,
      wallets: `${ROOTS.DASHBOARD}/admin/wallets`,
      cardCodes: `${ROOTS.DASHBOARD}/admin/card-codes`,
      recharges: `${ROOTS.DASHBOARD}/admin/recharges`,
      settings: `${ROOTS.DASHBOARD}/admin/settings`,
      translations: `${ROOTS.DASHBOARD}/admin/translations`,
    },
  },
};
