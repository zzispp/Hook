import type { SystemSettingsForm } from './system-settings-utils';

import { publicBaseUrlIsValid } from './system-settings-url-validation';

const MAX_API_ENDPOINT_FIELD_LENGTH = 255;

type T = (key: string, options?: Record<string, unknown>) => string;

export function validateApiEndpoints(form: SystemSettingsForm, t: T) {
  const invalid = form.api_endpoints.find((endpoint) => apiEndpointError(endpoint, t));
  return invalid ? apiEndpointError(invalid, t) : '';
}

function apiEndpointError(endpoint: SystemSettingsForm['api_endpoints'][number], t: T) {
  if (!validRequiredText(endpoint.id)) {
    return t('systemSettings.validation.apiEndpointIdRequired');
  }
  if (!validRequiredText(endpoint.name)) {
    return t('systemSettings.validation.apiEndpointNameRequired');
  }
  if (!validRequiredText(endpoint.url)) {
    return t('systemSettings.validation.apiEndpointUrlRequired');
  }
  if (endpoint.description.trim().length > MAX_API_ENDPOINT_FIELD_LENGTH) {
    return t('systemSettings.validation.apiEndpointDescriptionLength');
  }
  if (!publicBaseUrlIsValid(endpoint.url)) {
    return t('systemSettings.validation.apiEndpointUrlInvalid');
  }
  return '';
}

function validRequiredText(value: string) {
  const trimmed = value.trim();
  return Boolean(trimmed) && trimmed.length <= MAX_API_ENDPOINT_FIELD_LENGTH;
}
