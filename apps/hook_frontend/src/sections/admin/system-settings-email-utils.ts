import type { SystemSettingsForm } from './system-settings-utils';
import type { SystemSettingsSmtpTestRequest } from 'src/types/system-setting';

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

export function emailConfigComplete(form: SystemSettingsForm) {
  return Boolean(
    form.smtp_host.trim() &&
      Number(form.smtp_port || 0) >= MIN_SMTP_PORT &&
      form.smtp_username.trim() &&
      (form.smtp_password.trim() || form.smtp_password_set) &&
      form.smtp_from_email.trim()
  );
}

const MIN_SMTP_PORT = 1;
