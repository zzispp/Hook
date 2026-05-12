import type { SystemSettings, SystemSettingsUpdate } from 'src/types/system-setting';

export type SystemSettingsForm = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: string;
  request_record_payload_retention_days: string;
  request_record_level: SystemSettings['request_record_level'];
  max_request_body_size_kb: string;
  max_response_body_size_kb: string;
  sensitive_request_headers: string;
  record_request_headers: boolean;
  record_request_body: boolean;
  record_response_body: boolean;
  default_user_grant: string;
  default_rate_limit_rpm: string;
  currency: SystemSettings['currency'];
};

export const DEFAULT_SETTINGS_FORM: SystemSettingsForm = {
  site_name: '',
  site_subtitle: '',
  allow_registration: true,
  login_captcha_enabled: false,
  registration_captcha_enabled: false,
  auto_delete_expired_tokens: false,
  request_record_retention_days: '365',
  request_record_payload_retention_days: '30',
  request_record_level: 'basic',
  max_request_body_size_kb: '5120',
  max_response_body_size_kb: '5120',
  sensitive_request_headers: 'authorization, x-api-key, api-key, cookie, set-cookie',
  record_request_headers: false,
  record_request_body: false,
  record_response_body: false,
  default_user_grant: '0',
  default_rate_limit_rpm: '0',
  currency: 'USD',
};

export function formFromSettings(settings: SystemSettings): SystemSettingsForm {
  return {
    site_name: settings.site_name,
    site_subtitle: settings.site_subtitle,
    allow_registration: settings.allow_registration,
    login_captcha_enabled: settings.login_captcha_enabled,
    registration_captcha_enabled: settings.registration_captcha_enabled,
    auto_delete_expired_tokens: settings.auto_delete_expired_tokens,
    request_record_retention_days: String(settings.request_record_retention_days),
    request_record_payload_retention_days: String(settings.request_record_payload_retention_days),
    request_record_level: settings.request_record_level,
    max_request_body_size_kb: String(settings.max_request_body_size_kb),
    max_response_body_size_kb: String(settings.max_response_body_size_kb),
    sensitive_request_headers: settings.sensitive_request_headers,
    record_request_headers: settings.record_request_headers,
    record_request_body: settings.record_request_body,
    record_response_body: settings.record_response_body,
    default_user_grant: String(settings.default_user_grant),
    default_rate_limit_rpm: String(settings.default_rate_limit_rpm),
    currency: settings.currency,
  };
}

export function settingsPayload(form: SystemSettingsForm): SystemSettingsUpdate {
  return {
    site_name: form.site_name,
    site_subtitle: form.site_subtitle,
    allow_registration: form.allow_registration,
    login_captcha_enabled: form.login_captcha_enabled,
    registration_captcha_enabled: form.registration_captcha_enabled,
    auto_delete_expired_tokens: form.auto_delete_expired_tokens,
    request_record_retention_days: Number(form.request_record_retention_days || 0),
    request_record_payload_retention_days: Number(form.request_record_payload_retention_days || 0),
    request_record_level: form.request_record_level,
    max_request_body_size_kb: Number(form.max_request_body_size_kb || 0),
    max_response_body_size_kb: Number(form.max_response_body_size_kb || 0),
    sensitive_request_headers: form.sensitive_request_headers,
    record_request_headers: form.record_request_headers,
    record_request_body: form.record_request_body,
    record_response_body: form.record_response_body,
    default_user_grant: Number(form.default_user_grant || 0),
    default_rate_limit_rpm: Number(form.default_rate_limit_rpm || 0),
    currency: form.currency,
  };
}
