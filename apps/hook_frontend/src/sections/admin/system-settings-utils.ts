import type { SystemSettings, SystemSettingsUpdate } from 'src/types/system-setting';

export type SystemSettingsForm = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_retention_days: string;
  request_record_payload_retention_days: string;
  default_user_grant: string;
  default_rate_limit_rpm: string;
  currency: SystemSettings['currency'];
};

export const DEFAULT_SETTINGS_FORM: SystemSettingsForm = {
  site_name: '',
  site_subtitle: '',
  allow_registration: true,
  auto_delete_expired_tokens: false,
  request_record_retention_days: '365',
  request_record_payload_retention_days: '30',
  default_user_grant: '0',
  default_rate_limit_rpm: '0',
  currency: 'USD',
};

export function formFromSettings(settings: SystemSettings): SystemSettingsForm {
  return {
    site_name: settings.site_name,
    site_subtitle: settings.site_subtitle,
    allow_registration: settings.allow_registration,
    auto_delete_expired_tokens: settings.auto_delete_expired_tokens,
    request_record_retention_days: String(settings.request_record_retention_days),
    request_record_payload_retention_days: String(settings.request_record_payload_retention_days),
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
    auto_delete_expired_tokens: form.auto_delete_expired_tokens,
    request_record_retention_days: Number(form.request_record_retention_days || 0),
    request_record_payload_retention_days: Number(form.request_record_payload_retention_days || 0),
    default_user_grant: Number(form.default_user_grant || 0),
    default_rate_limit_rpm: Number(form.default_rate_limit_rpm || 0),
    currency: form.currency,
  };
}
