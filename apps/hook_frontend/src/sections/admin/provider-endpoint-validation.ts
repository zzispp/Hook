import type { AdminT } from './shared';
import type { EndpointEditState } from './provider-endpoint-rule-types';

import { normalizeBaseUrl } from './provider-management-utils';

const MAX_PORT = 65535;
const BASE_URL_PATTERN =
  /^https?:\/\/(?:localhost|[A-Za-z0-9.-]+|\[[0-9A-Fa-f:.]+\])(?::[0-9]{1,5})?(?:\/[A-Za-z0-9._~%!$&'()*+,;=:@/-]*)?$/;
const CUSTOM_PATH_PATTERN = /^\/[A-Za-z0-9._~%!$&'()*+,;=:@/{}-]*$/;

export function validateEndpointUrlFields(state: EndpointEditState, t: AdminT) {
  if (!normalizeBaseUrl(state.baseUrl)) return t('providers.baseUrlRequired');
  if (!isValidEndpointBaseUrl(state.baseUrl)) return t('providers.invalidBaseUrl');
  if (!isValidEndpointCustomPath(state.customPath)) return t('providers.invalidCustomPath');
  return null;
}

export function isValidEndpointBaseUrl(value: string) {
  const normalized = normalizeBaseUrl(value);
  if (!BASE_URL_PATTERN.test(normalized)) return false;

  try {
    const url = new URL(normalized);
    return (
      ['http:', 'https:'].includes(url.protocol) &&
      Boolean(url.hostname) &&
      !url.username &&
      !url.password &&
      !url.search &&
      !url.hash &&
      validPort(url.port)
    );
  } catch {
    return false;
  }
}

function isValidEndpointCustomPath(value: string) {
  const trimmed = value.trim();
  return !trimmed || CUSTOM_PATH_PATTERN.test(trimmed);
}

function validPort(value: string) {
  if (!value) return true;
  const port = Number(value);
  return Number.isInteger(port) && port > 0 && port <= MAX_PORT;
}
