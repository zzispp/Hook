export const DASHBOARD_MENU_CODES = {
  apiManagement: 'admin_apis',
  apiTokens: 'api_tokens',
  billingGroups: 'admin_groups',
  dashboard: 'dashboard_home',
  menuManagement: 'admin_menus',
  modelCatalog: 'dashboard_models',
  modelManagement: 'admin_models',
  roleManagement: 'admin_roles',
  systemSettings: 'admin_settings',
  translationManagement: 'admin_translations',
  tokenManagement: 'admin_tokens',
  userManagement: 'admin_users',
  walletCenter: 'wallet_center',
  walletManagement: 'admin_wallets',
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
