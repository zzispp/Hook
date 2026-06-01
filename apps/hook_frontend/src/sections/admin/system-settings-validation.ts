import type { UserGroup } from 'src/types/user-group';
import type { PaymentChannel } from 'src/types/recharge';
import type { SystemSettingsForm } from './system-settings-utils';

import { emailConfigComplete } from './system-settings-utils';
import { publicBaseUrlIsValid } from './system-settings-url-validation';
import { validateContactMethods } from './system-settings-contact-methods-validation';
import { validateAuthProviderFields } from './system-settings-auth-provider-validation';

const MAX_SITE_NAME_LENGTH = 100;
const MAX_SITE_SUBTITLE_LENGTH = 200;
const MAX_PUBLIC_BASE_URL_LENGTH = 255;
const MAX_SMTP_HOST_LENGTH = 255;
const MAX_SMTP_USERNAME_LENGTH = 255;
const MAX_SMTP_PASSWORD_LENGTH = 1024;
const MAX_SMTP_FROM_EMAIL_LENGTH = 255;
const MAX_SMTP_FROM_NAME_LENGTH = 100;
const MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH = 200;
const MIN_SMTP_PORT = 1;
const MAX_SMTP_PORT = 65535;
const CODE_PATTERN = /^[A-Za-z0-9_-]+$/;
const HEADER_TOKEN_PATTERN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;

type T = (key: string, options?: Record<string, unknown>) => string;

export type SystemSettingsValidationContext = {
  paymentChannels?: PaymentChannel[];
  paymentChannelsLoading?: boolean;
  paymentChannelsError?: Error;
  userGroups?: UserGroup[];
  userGroupsTotal?: number;
  userGroupsLoading?: boolean;
  userGroupsError?: Error;
};

export function validateSystemSettingsBeforeSubmit(
  form: SystemSettingsForm,
  context: SystemSettingsValidationContext,
  t: T
) {
  return (
    validateSiteFields(form, t) ||
    validateContactMethods(form, t) ||
    validateDefaultUserGroup(form, context, t) ||
    validateNumberFields(form, t) ||
    validateRequestRecordFields(form, t) ||
    validateRechargeFields(form, context, t) ||
    validateAuthProviderFields(form, t) ||
    validateEmailFields(form, t)
  );
}

function validateSiteFields(form: SystemSettingsForm, t: T) {
  if (!form.site_name.trim() || form.site_name.trim().length > MAX_SITE_NAME_LENGTH) {
    return t('systemSettings.validation.siteNameLength', { max: MAX_SITE_NAME_LENGTH });
  }
  if (form.site_subtitle.trim().length > MAX_SITE_SUBTITLE_LENGTH) {
    return t('systemSettings.validation.siteSubtitleLength', { max: MAX_SITE_SUBTITLE_LENGTH });
  }
  if (form.public_base_url.trim().length > MAX_PUBLIC_BASE_URL_LENGTH) {
    return t('systemSettings.validation.publicBaseUrlLength', { max: MAX_PUBLIC_BASE_URL_LENGTH });
  }
  if (form.public_base_url.trim() && !publicBaseUrlIsValid(form.public_base_url)) {
    return t('systemSettings.validation.publicBaseUrlInvalid');
  }
  return '';
}

function validateDefaultUserGroup(
  form: SystemSettingsForm,
  context: SystemSettingsValidationContext,
  t: T
) {
  const code = form.default_user_group_code.trim();
  if (!code || code.length > 64 || !CODE_PATTERN.test(code)) {
    return t('systemSettings.validation.defaultUserGroupCodeInvalid');
  }
  if (context.userGroupsLoading) {
    return t('systemSettings.validation.userGroupsLoadingBeforeSave');
  }
  if (context.userGroupsError) {
    return t('systemSettings.validation.userGroupsUnavailableBeforeSave');
  }
  const userGroups = context.userGroups ?? [];
  const userGroupFound = userGroups.some((group) => group.code === code && group.is_active);
  const userGroupsComplete = context.userGroupsTotal === undefined || context.userGroupsTotal <= userGroups.length;
  if (!userGroupFound && userGroupsComplete) {
    return t('systemSettings.validation.defaultUserGroupNotActive');
  }
  return '';
}

function validateNumberFields(form: SystemSettingsForm, t: T) {
  if (decimalValue(form.default_user_grant) === null || decimalValue(form.default_user_grant)! < 0) {
    return t('systemSettings.validation.defaultUserGrantNonNegative');
  }
  if (integerValue(form.default_rate_limit_rpm) === null || integerValue(form.default_rate_limit_rpm)! < 0) {
    return t('systemSettings.validation.defaultRateLimitRpmNonNegative');
  }
  if (integerValue(form.token_limit_per_user) === null || integerValue(form.token_limit_per_user)! <= 0) {
    return t('systemSettings.validation.tokenLimitPerUserPositive');
  }
  return '';
}

function validateRequestRecordFields(form: SystemSettingsForm, t: T) {
  const sizeFields = [
    form.client_max_request_body_size_kb,
    form.client_max_response_body_size_kb,
    form.provider_max_request_body_size_kb,
    form.provider_max_response_body_size_kb,
  ];
  if (sizeFields.some((value) => integerValue(value) === null || integerValue(value)! <= 0)) {
    return t('systemSettings.validation.requestRecordBodyLimitPositive');
  }
  return (
    validateHeaderList(form.client_sensitive_request_headers, t) ||
    validateHeaderList(form.provider_sensitive_request_headers, t)
  );
}

function validateRechargeFields(
  form: SystemSettingsForm,
  context: SystemSettingsValidationContext,
  t: T
) {
  const ratio = decimalValue(form.recharge_arrival_ratio);
  const expireMinutes = integerValue(form.recharge_order_expire_minutes);
  const maxUnpaidOrders = integerValue(form.recharge_max_unpaid_orders);
  const minAmount = decimalValue(form.recharge_min_amount);
  const maxAmount = decimalValue(form.recharge_max_amount);
  if (ratio === null || ratio <= 0) {
    return t('systemSettings.validation.rechargeArrivalRatioPositive');
  }
  if (expireMinutes === null || expireMinutes <= 0) {
    return t('systemSettings.validation.rechargeOrderExpireMinutesPositive');
  }
  if (maxUnpaidOrders === null || maxUnpaidOrders <= 0) {
    return t('systemSettings.validation.rechargeMaxUnpaidOrdersPositive');
  }
  if (minAmount === null || minAmount <= 0 || maxAmount === null || maxAmount <= 0) {
    return t('systemSettings.validation.rechargeAmountPositive');
  }
  if (minAmount > maxAmount) {
    return t('systemSettings.validation.rechargeMinAmountNotGreaterThanMax');
  }
  if (!form.recharge_enabled) {
    return '';
  }
  if (context.paymentChannelsLoading) {
    return t('systemSettings.recharge.paymentChannelsLoadingBeforeEnablingRecharge');
  }
  if (context.paymentChannelsError) {
    return t('systemSettings.recharge.paymentChannelsUnavailableBeforeEnablingRecharge');
  }
  if (!hasReadyPaymentChannel(context.paymentChannels ?? [])) {
    return t('systemSettings.recharge.readyPaymentChannelRequiredBeforeEnablingRecharge');
  }
  return '';
}

function validateEmailFields(form: SystemSettingsForm, t: T) {
  if (form.smtp_host.trim().length > MAX_SMTP_HOST_LENGTH) {
    return t('systemSettings.validation.smtpHostLength', { max: MAX_SMTP_HOST_LENGTH });
  }
  if (form.smtp_username.trim().length > MAX_SMTP_USERNAME_LENGTH) {
    return t('systemSettings.validation.smtpUsernameLength', { max: MAX_SMTP_USERNAME_LENGTH });
  }
  if (form.smtp_password.trim().length > MAX_SMTP_PASSWORD_LENGTH) {
    return t('systemSettings.validation.smtpPasswordLength', { max: MAX_SMTP_PASSWORD_LENGTH });
  }
  if (form.smtp_from_email.trim().length > MAX_SMTP_FROM_EMAIL_LENGTH) {
    return t('systemSettings.validation.smtpFromEmailLength', { max: MAX_SMTP_FROM_EMAIL_LENGTH });
  }
  if (form.smtp_from_name.trim().length > MAX_SMTP_FROM_NAME_LENGTH) {
    return t('systemSettings.validation.smtpFromNameLength', { max: MAX_SMTP_FROM_NAME_LENGTH });
  }
  return validateSmtpPort(form, t) || validateSmtpEmail(form, t) || validateEmailPrerequisites(form, t);
}

function validateSmtpPort(form: SystemSettingsForm, t: T) {
  const port = integerValue(form.smtp_port);
  if (port === null || port < MIN_SMTP_PORT || port > MAX_SMTP_PORT) {
    return t('systemSettings.validation.smtpPortRange', {
      min: MIN_SMTP_PORT,
      max: MAX_SMTP_PORT,
    });
  }
  return '';
}

function validateSmtpEmail(form: SystemSettingsForm, t: T) {
  if (form.smtp_from_email.trim() && !emailAddressIsValid(form.smtp_from_email)) {
    return t('systemSettings.validation.smtpFromEmailInvalid');
  }
  if (form.email_suffix_mode !== 'none' && !form.email_suffixes.trim()) {
    return t('systemSettings.validation.emailSuffixesRequired');
  }
  const invalidSuffix = emailSuffixes(form.email_suffixes).find((suffix) => !emailSuffixIsValid(suffix));
  if (invalidSuffix) {
    return t('systemSettings.validation.emailSuffixInvalid', { suffix: invalidSuffix });
  }
  return validateTemplate('registration', form.email_template_registration_subject, form.email_template_registration_html, t)
    || validateTemplate('passwordReset', form.email_template_password_reset_subject, form.email_template_password_reset_html, t);
}

function validateEmailPrerequisites(form: SystemSettingsForm, t: T) {
  const emailReady = form.email_config_enabled && emailConfigComplete(form);
  if (form.registration_email_verification_enabled && !emailReady) {
    return t('systemSettings.helper.registrationEmailVerificationRequiresEmailConfig');
  }
  if (form.password_reset_enabled && !emailReady) {
    return t('systemSettings.helper.passwordResetRequiresEmailConfig');
  }
  if (form.support_ticket_email_notifications_enabled && !emailReady) {
    return t('systemSettings.helper.supportTicketEmailNotificationsRequiresEmailConfig');
  }
  return '';
}

function validateTemplate(type: 'registration' | 'passwordReset', subject: string, html: string, t: T) {
  const suffix = type === 'registration' ? 'Registration' : 'PasswordReset';
  if (!subject.trim() || subject.trim().length > MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH) {
    return t(`systemSettings.validation.emailTemplate${suffix}SubjectLength`, {
      max: MAX_EMAIL_TEMPLATE_SUBJECT_LENGTH,
    });
  }
  if (!html.trim()) {
    return t(`systemSettings.validation.emailTemplate${suffix}HtmlRequired`);
  }
  return '';
}

function validateHeaderList(value: string, t: T) {
  const invalidHeader = value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
    .find((header) => !HEADER_TOKEN_PATTERN.test(header));
  return invalidHeader
    ? t('systemSettings.validation.sensitiveRequestHeaderInvalid', { header: invalidHeader })
    : '';
}

function hasReadyPaymentChannel(channels: PaymentChannel[]) {
  return channels.some((channel) => channel.enabled && channel.secret_set);
}

function decimalValue(value: string) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function integerValue(value: string) {
  const number = Number(value);
  return Number.isInteger(number) ? number : null;
}

function emailAddressIsValid(value: string) {
  const trimmed = value.trim();
  const parts = trimmed.split('@');
  return parts.length === 2 && Boolean(parts[0]) && parts[1].includes('.') && Boolean(parts[1]);
}

function emailSuffixes(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function emailSuffixIsValid(value: string) {
  return !value.includes('@') && value.includes('.');
}
