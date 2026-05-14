import type { ProviderSchedulingMode } from './provider';

export type DisplayCurrency = 'USD' | 'CNY';
export type RequestRecordLevel = 'basic' | 'headers' | 'full';
export type SmtpEncryption = 'none' | 'tls' | 'ssl';
export type EmailSuffixMode = 'none' | 'whitelist' | 'blacklist';

export type SystemSettings = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
  registration_email_verification_enabled: boolean;
  email_config_enabled: boolean;
  support_ticket_email_notifications_enabled: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: number;
  request_record_payload_retention_days: number;
  request_record_level: RequestRecordLevel;
  max_request_body_size_kb: number;
  max_response_body_size_kb: number;
  sensitive_request_headers: string;
  record_request_headers: boolean;
  record_request_body: boolean;
  record_response_body: boolean;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
  currency: DisplayCurrency;
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password_set: boolean;
  smtp_from_email: string;
  smtp_from_name: string;
  smtp_encryption: SmtpEncryption;
  email_suffix_mode: EmailSuffixMode;
  email_suffixes: string;
  email_template_registration_subject: string;
  email_template_registration_html: string;
  email_template_password_reset_subject: string;
  email_template_password_reset_html: string;
  created_at: string;
  updated_at: string;
};

export type SystemSettingsUpdate = Partial<{
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
  registration_email_verification_enabled: boolean;
  email_config_enabled: boolean;
  support_ticket_email_notifications_enabled: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: number;
  request_record_payload_retention_days: number;
  request_record_level: RequestRecordLevel;
  max_request_body_size_kb: number;
  max_response_body_size_kb: number;
  sensitive_request_headers: string;
  record_request_headers: boolean;
  record_request_body: boolean;
  record_response_body: boolean;
  default_user_grant: number;
  default_rate_limit_rpm: number;
  scheduling_mode: ProviderSchedulingMode;
  currency: DisplayCurrency;
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
  smtp_from_email: string;
  smtp_from_name: string;
  smtp_encryption: SmtpEncryption;
  email_suffix_mode: EmailSuffixMode;
  email_suffixes: string;
  email_template_registration_subject: string;
  email_template_registration_html: string;
  email_template_password_reset_subject: string;
  email_template_password_reset_html: string;
}>;

export type SystemSettingsSmtpTestRequest = Partial<{
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
  smtp_from_email: string;
  smtp_from_name: string;
  smtp_encryption: SmtpEncryption;
}>;

export type SystemSettingsSmtpTestResponse = {
  success: boolean;
  message: string;
};

export type ExchangeRateResponse = {
  base: string;
  target: string;
  rate: number;
  source: string;
  source_date: string;
  updated_at: string;
};

export type CurrencyDisplayResponse = {
  currency: DisplayCurrency;
  usd_cny_rate?: ExchangeRateResponse | null;
};
