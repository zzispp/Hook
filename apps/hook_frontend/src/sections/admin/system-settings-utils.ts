import type {
  SystemSettings,
  SystemSettingsUpdate,
  SystemSettingsSmtpTestRequest,
} from 'src/types/system-setting';

export type SystemSettingsForm = {
  site_name: string;
  site_subtitle: string;
  allow_registration: boolean;
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
  registration_email_verification_enabled: boolean;
  email_config_enabled: boolean;
  support_ticket_email_notifications_enabled: boolean;
  auto_delete_expired_tokens: boolean;
  request_record_cleanup_enabled: boolean;
  request_record_cleanup_interval_hours: string;
  performance_monitoring_cleanup_enabled: boolean;
  performance_monitoring_cleanup_interval_hours: string;
  request_record_retention_days: string;
  request_record_payload_retention_days: string;
  performance_monitoring_retention_days: string;
  client_request_record_level: SystemSettings['client_request_record_level'];
  client_max_request_body_size_kb: string;
  client_max_response_body_size_kb: string;
  client_sensitive_request_headers: string;
  provider_request_record_level: SystemSettings['provider_request_record_level'];
  provider_max_request_body_size_kb: string;
  provider_max_response_body_size_kb: string;
  provider_sensitive_request_headers: string;
  default_user_grant: string;
  default_rate_limit_rpm: string;
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

const DEFAULT_REGISTRATION_TEMPLATE_HTML = `<!DOCTYPE html>
<html lang="zh-CN">
<body style="margin:0;padding:0;background:#F9FAFB;font-family:'Public Sans',Arial,'Microsoft YaHei',sans-serif;color:#1C252E;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#F9FAFB;padding:32px 16px;">
    <tr>
      <td align="center">
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:560px;background:#FFFFFF;border:1px solid #DFE3E8;border-radius:8px;overflow:hidden;">
          <tr><td style="height:6px;background:#00A76F;"></td></tr>
          <tr>
            <td style="padding:32px;">
              <p style="margin:0 0 16px;color:#007867;font-size:13px;font-weight:700;letter-spacing:0;">{{app_name}}</p>
              <h1 style="margin:0 0 12px;color:#1C252E;font-size:24px;line-height:1.35;font-weight:700;">注册验证码</h1>
              <p style="margin:0 0 24px;color:#637381;font-size:15px;line-height:1.7;">请使用以下验证码完成邮箱验证。</p>
              <div style="padding:20px 16px;background:#C8FAD6;border:1px solid #5BE49B;border-radius:8px;text-align:center;">
                <span style="color:#004B50;font-size:36px;line-height:1.2;font-weight:700;letter-spacing:8px;">{{code}}</span>
              </div>
              <p style="margin:24px 0 0;color:#637381;font-size:14px;line-height:1.7;">验证码将在 {{expire_minutes}} 分钟后失效。</p>
              <p style="margin:8px 0 0;color:#919EAB;font-size:13px;line-height:1.7;">收件邮箱：{{email}}</p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>`;

const DEFAULT_PASSWORD_RESET_TEMPLATE_HTML = `<!DOCTYPE html>
<html lang="zh-CN">
<body style="margin:0;padding:0;background:#F9FAFB;font-family:'Public Sans',Arial,'Microsoft YaHei',sans-serif;color:#1C252E;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#F9FAFB;padding:32px 16px;">
    <tr>
      <td align="center">
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:560px;background:#FFFFFF;border:1px solid #DFE3E8;border-radius:8px;overflow:hidden;">
          <tr><td style="height:6px;background:#00A76F;"></td></tr>
          <tr>
            <td style="padding:32px;">
              <p style="margin:0 0 16px;color:#007867;font-size:13px;font-weight:700;letter-spacing:0;">{{app_name}}</p>
              <h1 style="margin:0 0 12px;color:#1C252E;font-size:24px;line-height:1.35;font-weight:700;">找回密码</h1>
              <p style="margin:0 0 24px;color:#637381;font-size:15px;line-height:1.7;">请点击下方按钮继续重置账户密码。</p>
              <p style="margin:0 0 24px;">
                <a href="{{reset_link}}" style="display:inline-block;padding:12px 22px;background:#00A76F;color:#FFFFFF;text-decoration:none;border-radius:8px;font-size:14px;font-weight:700;">重置密码</a>
              </p>
              <p style="margin:0 0 12px;color:#637381;font-size:14px;line-height:1.7;">链接将在 {{expire_minutes}} 分钟后失效。</p>
              <p style="margin:0;color:#919EAB;font-size:13px;line-height:1.7;">无法打开按钮时，请复制链接访问：{{reset_link}}</p>
              <p style="margin:8px 0 0;color:#919EAB;font-size:13px;line-height:1.7;">收件邮箱：{{email}}</p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>`;

export const DEFAULT_SETTINGS_FORM: SystemSettingsForm = {
  site_name: '',
  site_subtitle: '',
  allow_registration: true,
  login_captcha_enabled: false,
  registration_captcha_enabled: false,
  registration_email_verification_enabled: false,
  email_config_enabled: false,
  support_ticket_email_notifications_enabled: false,
  auto_delete_expired_tokens: false,
  request_record_cleanup_enabled: true,
  request_record_cleanup_interval_hours: '24',
  performance_monitoring_cleanup_enabled: true,
  performance_monitoring_cleanup_interval_hours: '24',
  request_record_retention_days: '365',
  request_record_payload_retention_days: '30',
  performance_monitoring_retention_days: '30',
  client_request_record_level: 'full',
  client_max_request_body_size_kb: '5120',
  client_max_response_body_size_kb: '5120',
  client_sensitive_request_headers: 'authorization, x-api-key, api-key, cookie, set-cookie',
  provider_request_record_level: 'full',
  provider_max_request_body_size_kb: '5120',
  provider_max_response_body_size_kb: '5120',
  provider_sensitive_request_headers: 'authorization, x-api-key, api-key, cookie, set-cookie',
  default_user_grant: '0',
  default_rate_limit_rpm: '0',
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
    allow_registration: settings.allow_registration,
    login_captcha_enabled: settings.login_captcha_enabled,
    registration_captcha_enabled: settings.registration_captcha_enabled,
    registration_email_verification_enabled: settings.registration_email_verification_enabled,
    email_config_enabled: settings.email_config_enabled,
    support_ticket_email_notifications_enabled:
      settings.support_ticket_email_notifications_enabled,
    auto_delete_expired_tokens: settings.auto_delete_expired_tokens,
    request_record_cleanup_enabled: settings.request_record_cleanup_enabled,
    request_record_cleanup_interval_hours: String(settings.request_record_cleanup_interval_hours),
    performance_monitoring_cleanup_enabled: settings.performance_monitoring_cleanup_enabled,
    performance_monitoring_cleanup_interval_hours: String(
      settings.performance_monitoring_cleanup_interval_hours
    ),
    request_record_retention_days: String(settings.request_record_retention_days),
    request_record_payload_retention_days: String(settings.request_record_payload_retention_days),
    performance_monitoring_retention_days: String(
      settings.performance_monitoring_retention_days
    ),
    client_request_record_level: settings.client_request_record_level,
    client_max_request_body_size_kb: String(settings.client_max_request_body_size_kb),
    client_max_response_body_size_kb: String(settings.client_max_response_body_size_kb),
    client_sensitive_request_headers: settings.client_sensitive_request_headers,
    provider_request_record_level: settings.provider_request_record_level,
    provider_max_request_body_size_kb: String(settings.provider_max_request_body_size_kb),
    provider_max_response_body_size_kb: String(settings.provider_max_response_body_size_kb),
    provider_sensitive_request_headers: settings.provider_sensitive_request_headers,
    default_user_grant: String(settings.default_user_grant),
    default_rate_limit_rpm: String(settings.default_rate_limit_rpm),
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
    allow_registration: form.allow_registration,
    login_captcha_enabled: form.login_captcha_enabled,
    registration_captcha_enabled: form.registration_captcha_enabled,
    registration_email_verification_enabled: form.registration_email_verification_enabled,
    email_config_enabled: form.email_config_enabled,
    support_ticket_email_notifications_enabled:
      form.support_ticket_email_notifications_enabled,
    auto_delete_expired_tokens: form.auto_delete_expired_tokens,
    request_record_cleanup_enabled: form.request_record_cleanup_enabled,
    request_record_cleanup_interval_hours: Number(form.request_record_cleanup_interval_hours || 0),
    performance_monitoring_cleanup_enabled: form.performance_monitoring_cleanup_enabled,
    performance_monitoring_cleanup_interval_hours: Number(
      form.performance_monitoring_cleanup_interval_hours || 0
    ),
    request_record_retention_days: Number(form.request_record_retention_days || 0),
    request_record_payload_retention_days: Number(form.request_record_payload_retention_days || 0),
    performance_monitoring_retention_days: Number(
      form.performance_monitoring_retention_days || 0
    ),
    client_request_record_level: form.client_request_record_level,
    client_max_request_body_size_kb: Number(form.client_max_request_body_size_kb || 0),
    client_max_response_body_size_kb: Number(form.client_max_response_body_size_kb || 0),
    client_sensitive_request_headers: form.client_sensitive_request_headers,
    provider_request_record_level: form.provider_request_record_level,
    provider_max_request_body_size_kb: Number(form.provider_max_request_body_size_kb || 0),
    provider_max_response_body_size_kb: Number(form.provider_max_response_body_size_kb || 0),
    provider_sensitive_request_headers: form.provider_sensitive_request_headers,
    default_user_grant: Number(form.default_user_grant || 0),
    default_rate_limit_rpm: Number(form.default_rate_limit_rpm || 0),
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
