// ----------------------------------------------------------------------

export type ApiEnvelope<T> = {
  success: boolean;
  message: string;
  data?: T;
};

export type PageResponse<T> = {
  items: T[];
  total: number;
  page: number;
  page_size: number;
};

export type Role = {
  code: string;
  name: string;
  description: string;
  enabled: boolean;
  system: boolean;
  sort_order: number;
};

export type RoleInput = {
  code: string;
  name: string;
  description: string;
  enabled: boolean;
  sort_order: number;
};

export type ApiPermission = {
  id: string;
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  enabled: boolean;
  system: boolean;
  menu_item_ids: string[];
};

export type ApiPermissionInput = {
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  enabled: boolean;
  menu_item_ids: string[];
};

export type MenuSection = {
  id: string;
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuSectionInput = {
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuItem = {
  id: string;
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};

export type MenuItemInput = {
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};

export type MenuApiBinding = {
  api_permission_ids: string[];
};

export type ApiMenuBinding = {
  menu_item_ids: string[];
};

export type RolePermissionBinding = {
  menu_item_ids: string[];
  api_permission_ids: string[];
  readonly_apis: ApiPermission[];
};

export type NavResponse = {
  nav_items: BackendNavSection[];
};

export type BackendNavSection = {
  code: string;
  subheader: string;
  items: BackendNavItem[];
};

export type BackendNavItem = {
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  children: BackendNavItem[];
};

export type SystemUser = {
  id: string;
  username: string;
  email: string;
  role: string;
  group_code: string;
  is_active: boolean;
  allowed_model_ids: string[];
  allowed_provider_ids: string[];
  auth_source: string;
  email_verified: boolean;
  system: boolean;
  rate_limit_rpm?: number | null;
  quota_mode: UserQuotaMode;
  created_at: string;
  last_login_at?: string | null;
  wallet?: UserWalletSummary | null;
};

export type UserInput = {
  username: string;
  password: string;
  email: string;
  role: string;
  group_code: string;
  is_active: boolean;
  allowed_model_ids: string[];
  allowed_provider_ids: string[];
  rate_limit_rpm?: number | null;
  quota_mode: UserQuotaMode;
};

export type UserQuotaMode = 'wallet' | 'unlimited';

export type UserWalletSummary = {
  id: string;
  available_balance: number;
  recharge_balance: number;
  gift_balance: number;
  total_consumed: number;
  currency: string;
  status: string;
};
