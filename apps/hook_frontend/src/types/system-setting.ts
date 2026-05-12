import type { ProviderSchedulingMode } from './provider';

export type DisplayCurrency = 'USD' | 'CNY';

export type SystemSettings = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: number;
  request_record_payload_retention_days: number;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
  currency: DisplayCurrency;
  created_at: string;
  updated_at: string;
};

export type SystemSettingsUpdate = Partial<{
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: number;
  request_record_payload_retention_days: number;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
  currency: DisplayCurrency;
}>;

export type ExchangeRateResponse = {
  base: string;
  target: string;
  rate: number;
  source: string;
  source_date: string;
  updated_at: string;
};
