import type { SystemSettings, SystemSettingsUpdate } from 'src/types/system-setting';

export type SystemSettingsForm = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  auto_delete_expired_tokens: boolean;
  default_user_grant: string;
  default_rate_limit_rpm: string;
  scheduling_mode: SystemSettings['scheduling_mode'];
};

export const DEFAULT_SETTINGS_FORM: SystemSettingsForm = {
  site_name: '',
  site_subtitle: '',
  allow_registration: true,
  auto_delete_expired_tokens: false,
  default_user_grant: '0',
  default_rate_limit_rpm: '0',
  scheduling_mode: 'cache_affinity',
};

export function formFromSettings(settings: SystemSettings): SystemSettingsForm {
  return {
    site_name: settings.site_name,
    site_subtitle: settings.site_subtitle,
    allow_registration: settings.allow_registration,
    auto_delete_expired_tokens: settings.auto_delete_expired_tokens,
    default_user_grant: String(settings.default_user_grant),
    default_rate_limit_rpm: String(settings.default_rate_limit_rpm),
    scheduling_mode: settings.scheduling_mode,
  };
}

export function settingsPayload(form: SystemSettingsForm): SystemSettingsUpdate {
  return {
    site_name: form.site_name,
    site_subtitle: form.site_subtitle,
    allow_registration: form.allow_registration,
    auto_delete_expired_tokens: form.auto_delete_expired_tokens,
    default_user_grant: Number(form.default_user_grant || 0),
    default_rate_limit_rpm: Number(form.default_rate_limit_rpm || 0),
    scheduling_mode: form.scheduling_mode,
  };
}
