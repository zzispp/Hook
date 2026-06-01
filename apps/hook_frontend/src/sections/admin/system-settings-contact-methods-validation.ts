import type { SystemSettingsForm } from './system-settings-utils';

import { publicBaseUrlIsValid } from './system-settings-url-validation';

const MAX_CONTACT_FIELD_LENGTH = 255;
const MAX_CONTACT_QR_CODE_URL_LENGTH = 4096;
const MAX_CONTACT_QR_CODE_DATA_URL_LENGTH = 1048576;

type T = (key: string, options?: Record<string, unknown>) => string;

export function validateContactMethods(form: SystemSettingsForm, t: T) {
  const invalid = form.contact_methods.find((method) => contactMethodError(method, t));
  return invalid ? contactMethodError(invalid, t) : '';
}

function contactMethodError(method: SystemSettingsForm['contact_methods'][number], t: T) {
  if (!validRequiredText(method.id, MAX_CONTACT_FIELD_LENGTH)) {
    return t('systemSettings.validation.contactIdRequired');
  }
  if (!validRequiredText(method.icon, MAX_CONTACT_FIELD_LENGTH)) {
    return t('systemSettings.validation.contactIconRequired');
  }
  if (!validRequiredText(method.value, MAX_CONTACT_FIELD_LENGTH)) {
    return t('systemSettings.validation.contactValueRequired');
  }
  if (method.type === 'custom' && !validRequiredText(method.custom_type, MAX_CONTACT_FIELD_LENGTH)) {
    return t('systemSettings.validation.contactCustomTypeRequired');
  }
  if (!contactQrCodeIsValid(method.qr_code)) {
    return t('systemSettings.validation.contactQrCodeInvalid');
  }
  return '';
}

function validRequiredText(value: string, max: number) {
  const trimmed = value.trim();
  return Boolean(trimmed) && trimmed.length <= max;
}

function contactQrCodeIsValid(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return true;
  }
  if (trimmed.startsWith('data:image/')) {
    return trimmed.length <= MAX_CONTACT_QR_CODE_DATA_URL_LENGTH;
  }
  if (trimmed.length > MAX_CONTACT_QR_CODE_URL_LENGTH) {
    return false;
  }
  return publicBaseUrlIsValid(trimmed);
}
