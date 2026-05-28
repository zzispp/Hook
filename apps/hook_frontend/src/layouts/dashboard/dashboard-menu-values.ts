export const DASHBOARD_MENU_CODES = {
  apiManagement: 'admin_apis',
  apiTokens: 'api_tokens',
  announcementManagement: 'admin_announcements',
  announcements: 'announcements',
  billingGroupCatalog: 'dashboard_groups',
  billingGroups: 'admin_groups',
  cacheMonitoring: 'admin_cache_monitoring',
  dashboard: 'dashboard_home',
  userStats: 'admin_user_stats',
  costAnalysis: 'admin_cost_analysis',
  menuManagement: 'admin_menus',
  modelCatalog: 'dashboard_models',
  modelManagement: 'admin_models',
  performanceMonitoring: 'admin_performance_monitoring',
  providerManagement: 'admin_providers',
  requestRecords: 'admin_request_records',
  roleManagement: 'admin_roles',
  scheduledTaskManagement: 'admin_scheduled_tasks',
  systemSettings: 'admin_settings',
  translationManagement: 'admin_translations',
  tokenManagement: 'admin_tokens',
  ticketManagement: 'admin_tickets',
  tickets: 'support_tickets',
  usageRecords: 'usage_records',
  userManagement: 'admin_users',
  userGroups: 'admin_user_groups',
  walletCenter: 'wallet_center',
  walletManagement: 'admin_wallets',
  cardCodeManagement: 'admin_card_codes',
  rechargeManagement: 'admin_recharges',
} as const;

export const DASHBOARD_SECTION_CODES = {
  operations: 'operations',
  overview: 'overview',
  systemManagement: 'system_management',
} as const;

export type DashboardMenuCode = (typeof DASHBOARD_MENU_CODES)[keyof typeof DASHBOARD_MENU_CODES];
export type DashboardSectionCode = (typeof DASHBOARD_SECTION_CODES)[keyof typeof DASHBOARD_SECTION_CODES];

export function navTranslationKey(code: DashboardMenuCode | DashboardSectionCode) {
  return `nav.${code}` as const;
}
