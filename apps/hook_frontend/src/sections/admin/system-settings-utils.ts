import type {
  SystemSettings,
  SystemSettingsUpdate,
  SystemSettingsSmtpTestRequest,
} from 'src/types/system-setting';

import {
  DEFAULT_REGISTRATION_TEMPLATE_HTML,
  DEFAULT_PASSWORD_RESET_TEMPLATE_HTML,
} from './system-settings-email-templates';
import {
  authProviderPayloadFields,
  DEFAULT_AUTH_PROVIDER_FORM,
  authProviderFormFromSettings,
  applyAuthProviderSecretPayload,
} from './system-settings-auth-provider-utils';

export type SystemSettingsForm = {
  site_name: string;
  site_subtitle: string;
  public_base_url: string;
  site_logo_base64: string;
  allow_registration: boolean;
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
  support_ticket_captcha_enabled: boolean;
  recharge_captcha_enabled: boolean;
  registration_email_verification_enabled: boolean;
  auth_github_enabled: boolean;
  auth_github_client_id: string;
  auth_github_client_secret: string;
  auth_github_client_secret_set: boolean;
  auth_google_enabled: boolean;
  auth_google_client_id: string;
  auth_google_client_secret: string;
  auth_google_client_secret_set: boolean;
  auth_evm_enabled: boolean;
  auth_evm_chain_ids: string;
  auth_evm_statement: string;
  password_reset_enabled: boolean;
  email_config_enabled: boolean;
  support_ticket_email_notifications_enabled: boolean;
  default_user_group_code: string;
  token_limit_per_user: string;
  client_request_record_level: SystemSettings['client_request_record_level'];
  client_record_request_headers: boolean;
  client_record_request_body: boolean;
  client_record_response_headers: boolean;
  client_record_response_body: boolean;
  client_max_request_body_size_kb: string;
  client_max_response_body_size_kb: string;
  client_sensitive_request_headers: string;
  provider_request_record_level: SystemSettings['provider_request_record_level'];
  provider_record_request_headers: boolean;
  provider_record_request_body: boolean;
  provider_record_response_headers: boolean;
  provider_record_response_body: boolean;
  provider_max_request_body_size_kb: string;
  provider_max_response_body_size_kb: string;
  provider_sensitive_request_headers: string;
  default_user_grant: string;
  default_rate_limit_rpm: string;
  recharge_enabled: boolean;
  recharge_arrival_ratio: string;
  recharge_order_expire_minutes: string;
  recharge_max_unpaid_orders: string;
  recharge_min_amount: string;
  recharge_max_amount: string;
  smtp_host: string;
  smtp_port: string;
  smtp_username: string;
  smtp_password: string;
  smtp_password_set: boolean;
  smtp_from_email: string;
  smtp_from_name: string;
  smtp_encryption: SystemSettings['smtp_encryption'];
  email_suffix_mode: SystemSettings['email_suffix_mode'];
  email_suffixes: string;
  email_template_registration_subject: string;
  email_template_registration_html: string;
  email_template_password_reset_subject: string;
  email_template_password_reset_html: string;
};

export const DEFAULT_SETTINGS_FORM: SystemSettingsForm = {
  site_name: '',
  site_subtitle: '',
  public_base_url: '',
  site_logo_base64: '',
  allow_registration: true,
  login_captcha_enabled: false,
  registration_captcha_enabled: false,
  support_ticket_captcha_enabled: true,
  recharge_captcha_enabled: false,
  registration_email_verification_enabled: false,
  ...DEFAULT_AUTH_PROVIDER_FORM,
  password_reset_enabled: false,
  email_config_enabled: false,
  support_ticket_email_notifications_enabled: false,
  default_user_group_code: 'default',
  token_limit_per_user: '5',
  client_request_record_level: 'full',
  client_record_request_headers: true,
  client_record_request_body: true,
  client_record_response_headers: true,
  client_record_response_body: true,
  client_max_request_body_size_kb: '5120',
  client_max_response_body_size_kb: '5120',
  client_sensitive_request_headers: 'authorization, x-api-key, api-key, cookie, set-cookie',
  provider_request_record_level: 'full',
  provider_record_request_headers: true,
  provider_record_request_body: true,
  provider_record_response_headers: true,
  provider_record_response_body: true,
  provider_max_request_body_size_kb: '5120',
  provider_max_response_body_size_kb: '5120',
  provider_sensitive_request_headers: 'authorization, x-api-key, api-key, cookie, set-cookie',
  default_user_grant: '0',
  default_rate_limit_rpm: '0',
  recharge_enabled: false,
  recharge_arrival_ratio: '1',
  recharge_order_expire_minutes: '15',
  recharge_max_unpaid_orders: '5',
  recharge_min_amount: '0.01',
  recharge_max_amount: '3000',
  smtp_host: '',
  smtp_port: '587',
  smtp_username: '',
  smtp_password: '',
  smtp_password_set: false,
  smtp_from_email: '',
  smtp_from_name: 'Hook',
  smtp_encryption: 'tls',
  email_suffix_mode: 'none',
  email_suffixes: '',
  email_template_registration_subject: '注册验证码',
  email_template_registration_html: DEFAULT_REGISTRATION_TEMPLATE_HTML,
  email_template_password_reset_subject: '找回密码',
  email_template_password_reset_html: DEFAULT_PASSWORD_RESET_TEMPLATE_HTML,
};

export function formFromSettings(settings: SystemSettings): SystemSettingsForm {
  return {
    site_name: settings.site_name,
    site_subtitle: settings.site_subtitle,
    public_base_url: settings.public_base_url,
    site_logo_base64: settings.site_logo_base64,
    allow_registration: settings.allow_registration,
    login_captcha_enabled: settings.login_captcha_enabled,
    registration_captcha_enabled: settings.registration_captcha_enabled,
    support_ticket_captcha_enabled: settings.support_ticket_captcha_enabled,
    recharge_captcha_enabled: settings.recharge_captcha_enabled,
    registration_email_verification_enabled: settings.registration_email_verification_enabled,
    ...authProviderFormFromSettings(settings),
    password_reset_enabled: settings.password_reset_enabled,
    email_config_enabled: settings.email_config_enabled,
    support_ticket_email_notifications_enabled: settings.support_ticket_email_notifications_enabled,
    default_user_group_code: settings.default_user_group_code,
    token_limit_per_user: String(settings.token_limit_per_user),
    client_request_record_level: settings.client_request_record_level,
    client_record_request_headers: settings.client_record_request_headers,
    client_record_request_body: settings.client_record_request_body,
    client_record_response_headers: settings.client_record_response_headers,
    client_record_response_body: settings.client_record_response_body,
    client_max_request_body_size_kb: String(settings.client_max_request_body_size_kb),
    client_max_response_body_size_kb: String(settings.client_max_response_body_size_kb),
    client_sensitive_request_headers: settings.client_sensitive_request_headers,
    provider_request_record_level: settings.provider_request_record_level,
    provider_record_request_headers: settings.provider_record_request_headers,
    provider_record_request_body: settings.provider_record_request_body,
    provider_record_response_headers: settings.provider_record_response_headers,
    provider_record_response_body: settings.provider_record_response_body,
    provider_max_request_body_size_kb: String(settings.provider_max_request_body_size_kb),
    provider_max_response_body_size_kb: String(settings.provider_max_response_body_size_kb),
    provider_sensitive_request_headers: settings.provider_sensitive_request_headers,
    default_user_grant: String(settings.default_user_grant),
    default_rate_limit_rpm: String(settings.default_rate_limit_rpm),
    recharge_enabled: settings.recharge_enabled,
    recharge_arrival_ratio: String(settings.recharge_arrival_ratio),
    recharge_order_expire_minutes: String(settings.recharge_order_expire_minutes),
    recharge_max_unpaid_orders: String(settings.recharge_max_unpaid_orders),
    recharge_min_amount: String(settings.recharge_min_amount),
    recharge_max_amount: String(settings.recharge_max_amount),
    smtp_host: settings.smtp_host,
    smtp_port: String(settings.smtp_port),
    smtp_username: settings.smtp_username,
    smtp_password: '',
    smtp_password_set: settings.smtp_password_set,
    smtp_from_email: settings.smtp_from_email,
    smtp_from_name: settings.smtp_from_name,
    smtp_encryption: settings.smtp_encryption,
    email_suffix_mode: settings.email_suffix_mode,
    email_suffixes: settings.email_suffixes,
    email_template_registration_subject: settings.email_template_registration_subject,
    email_template_registration_html: settings.email_template_registration_html,
    email_template_password_reset_subject: settings.email_template_password_reset_subject,
    email_template_password_reset_html: settings.email_template_password_reset_html,
  };
}

export function settingsPayload(form: SystemSettingsForm): SystemSettingsUpdate {
  const payload: SystemSettingsUpdate = {
    site_name: form.site_name,
    site_subtitle: form.site_subtitle,
    public_base_url: form.public_base_url,
    site_logo_base64: form.site_logo_base64,
    allow_registration: form.allow_registration,
    login_captcha_enabled: form.login_captcha_enabled,
    registration_captcha_enabled: form.registration_captcha_enabled,
    support_ticket_captcha_enabled: form.support_ticket_captcha_enabled,
    recharge_captcha_enabled: form.recharge_captcha_enabled,
    registration_email_verification_enabled: form.registration_email_verification_enabled,
    ...authProviderPayloadFields(form),
    password_reset_enabled: form.password_reset_enabled,
    email_config_enabled: form.email_config_enabled,
    support_ticket_email_notifications_enabled: form.support_ticket_email_notifications_enabled,
    default_user_group_code: form.default_user_group_code,
    token_limit_per_user: Number(form.token_limit_per_user || 0),
    client_request_record_level: form.client_request_record_level,
    client_record_request_headers: form.client_record_request_headers,
    client_record_request_body: form.client_record_request_body,
    client_record_response_headers: form.client_record_response_headers,
    client_record_response_body: form.client_record_response_body,
    client_max_request_body_size_kb: Number(form.client_max_request_body_size_kb || 0),
    client_max_response_body_size_kb: Number(form.client_max_response_body_size_kb || 0),
    client_sensitive_request_headers: form.client_sensitive_request_headers,
    provider_request_record_level: form.provider_request_record_level,
    provider_record_request_headers: form.provider_record_request_headers,
    provider_record_request_body: form.provider_record_request_body,
    provider_record_response_headers: form.provider_record_response_headers,
    provider_record_response_body: form.provider_record_response_body,
    provider_max_request_body_size_kb: Number(form.provider_max_request_body_size_kb || 0),
    provider_max_response_body_size_kb: Number(form.provider_max_response_body_size_kb || 0),
    provider_sensitive_request_headers: form.provider_sensitive_request_headers,
    default_user_grant: Number(form.default_user_grant || 0),
    default_rate_limit_rpm: Number(form.default_rate_limit_rpm || 0),
    recharge_enabled: form.recharge_enabled,
    recharge_arrival_ratio: Number(form.recharge_arrival_ratio || 0),
    recharge_order_expire_minutes: Number(form.recharge_order_expire_minutes || 0),
    recharge_max_unpaid_orders: Number(form.recharge_max_unpaid_orders || 0),
    recharge_min_amount: Number(form.recharge_min_amount || 0),
    recharge_max_amount: Number(form.recharge_max_amount || 0),
    smtp_host: form.smtp_host,
    smtp_port: Number(form.smtp_port || 0),
    smtp_username: form.smtp_username,
    smtp_from_email: form.smtp_from_email,
    smtp_from_name: form.smtp_from_name,
    smtp_encryption: form.smtp_encryption,
    email_suffix_mode: form.email_suffix_mode,
    email_suffixes: form.email_suffixes,
    email_template_registration_subject: form.email_template_registration_subject,
    email_template_registration_html: form.email_template_registration_html,
    email_template_password_reset_subject: form.email_template_password_reset_subject,
    email_template_password_reset_html: form.email_template_password_reset_html,
  };
  if (form.smtp_password.trim()) {
    payload.smtp_password = form.smtp_password.trim();
  }
  applyAuthProviderSecretPayload(payload, form);
  return payload;
}

export function smtpTestPayload(form: SystemSettingsForm): SystemSettingsSmtpTestRequest {
  const payload: SystemSettingsSmtpTestRequest = {
    smtp_host: form.smtp_host,
    smtp_port: Number(form.smtp_port || 0),
    smtp_username: form.smtp_username,
    smtp_from_email: form.smtp_from_email,
    smtp_from_name: form.smtp_from_name,
    smtp_encryption: form.smtp_encryption,
  };
  if (form.smtp_password.trim()) {
    payload.smtp_password = form.smtp_password.trim();
  }
  return payload;
}

const MIN_SMTP_PORT = 1;

export function emailConfigComplete(form: SystemSettingsForm) {
  return Boolean(
    form.smtp_host.trim() &&
    Number(form.smtp_port || 0) >= MIN_SMTP_PORT &&
    form.smtp_username.trim() &&
    (form.smtp_password.trim() || form.smtp_password_set) &&
    form.smtp_from_email.trim()
  );
}
