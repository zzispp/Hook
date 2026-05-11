import type { ProviderSchedulingMode } from './provider';

export type SystemSettings = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
  created_at: string;
  updated_at: string;
};

export type SystemSettingsUpdate = Partial<{
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
}>;
