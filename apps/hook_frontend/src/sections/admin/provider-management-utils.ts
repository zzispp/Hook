import type { AdminT } from './shared';
import type {
  Provider,
  ProviderType,
  ProviderApiKey,
  ProviderCreate,
  ProviderApiKeyUpdate,
  ProviderApiKeyCreate,
  ProviderModelBindingCreate,
} from 'src/types/provider';

export const PROVIDER_TYPE_OPTIONS: ProviderType[] = ['custom'];
export const DEFAULT_PROVIDER_MAX_RETRIES = 2;
export const DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS = 300;
export const DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS = 30;
export const DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS = 30;

export const API_FORMAT_OPTIONS = [
  'openai:chat',
  'openai:cli',
  'openai:compact',
  'openai_image',
  'openai_image_edit',
  'claude:chat',
  'gemini:cli',
];

const API_FORMAT_DEFAULT_PATHS: Record<string, string> = {
  'openai:chat': '/v1/chat/completions',
  'openai:cli': '/v1/responses',
  'openai:compact': '/v1/responses/compact',
  openai_image: '/v1/images/generations',
  openai_image_edit: '/v1/images/edits',
  'gemini:cli': '/v1beta/models/{model}:{action}',
  'claude:chat': '/v1/messages',
};

export type ProviderForm = {
  name: string;
  provider_type: ProviderType;
  max_retries: string;
  request_timeout_seconds: string;
  stream_first_byte_timeout_seconds: string;
  stream_idle_timeout_seconds: string;
  priority: string;
  keep_priority_on_conversion: boolean;
  enable_format_conversion: boolean;
  is_active: boolean;
};

export type ApiKeyForm = {
  name: string;
  api_key: string;
  api_formats: string[];
  allowed_model_ids: string[];
  note: string;
  internal_priority: string;
  rpm_limit: string;
  time_range_enabled: boolean;
  time_range_start: string;
  time_range_end: string;
  is_active: boolean;
};

export type ProviderModelForm = {
  global_model_id: string;
  provider_model_name: string;
};

export const DEFAULT_PROVIDER_FORM: ProviderForm = {
  name: '',
  provider_type: 'custom',
  max_retries: '',
  request_timeout_seconds: '',
  stream_first_byte_timeout_seconds: '',
  stream_idle_timeout_seconds: '',
  priority: '100',
  keep_priority_on_conversion: false,
  enable_format_conversion: true,
  is_active: true,
};

export const DEFAULT_API_KEY_FORM: ApiKeyForm = {
  name: '',
  api_key: '',
  api_formats: [],
  allowed_model_ids: [],
  note: '',
  internal_priority: '10',
  rpm_limit: '0',
  time_range_enabled: false,
  time_range_start: '',
  time_range_end: '',
  is_active: true,
};

export const DEFAULT_PROVIDER_MODEL_FORM: ProviderModelForm = {
  global_model_id: '',
  provider_model_name: '',
};

export function providerFormFromProvider(provider: Provider): ProviderForm {
  return {
    name: provider.name,
    provider_type: provider.provider_type,
    max_retries: optionalNumberText(provider.max_retries),
    request_timeout_seconds: optionalNumberText(provider.request_timeout_seconds),
    stream_first_byte_timeout_seconds: optionalNumberText(
      provider.stream_first_byte_timeout_seconds
    ),
    stream_idle_timeout_seconds: optionalNumberText(provider.stream_idle_timeout_seconds),
    priority: String(provider.priority),
    keep_priority_on_conversion: provider.keep_priority_on_conversion,
    enable_format_conversion: provider.enable_format_conversion,
    is_active: provider.is_active,
  };
}

export function providerPayload(form: ProviderForm): ProviderCreate {
  return {
    name: form.name,
    provider_type: form.provider_type,
    max_retries: optionalNumber(form.max_retries) ?? DEFAULT_PROVIDER_MAX_RETRIES,
    request_timeout_seconds:
      optionalNumber(form.request_timeout_seconds) ?? DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS,
    stream_first_byte_timeout_seconds:
      optionalNumber(form.stream_first_byte_timeout_seconds) ??
      DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS,
    stream_idle_timeout_seconds:
      optionalNumber(form.stream_idle_timeout_seconds) ?? DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS,
    priority: requiredNumber(form.priority),
    keep_priority_on_conversion: form.keep_priority_on_conversion,
    enable_format_conversion: form.enable_format_conversion,
    is_active: form.is_active,
  };
}

export function apiKeyPayload(form: ApiKeyForm): ProviderApiKeyCreate {
  return {
    name: form.name,
    api_key: form.api_key,
    api_formats: normalizeSelectedApiFormats(form.api_formats),
    allowed_model_ids: normalizeSelectedIds(form.allowed_model_ids),
    note: trimmedOrNull(form.note),
    internal_priority: requiredNumber(form.internal_priority),
    rpm_limit: optionalNumber(form.rpm_limit),
    time_range_enabled: form.time_range_enabled,
    time_range_start: form.time_range_enabled ? trimmedOrNull(form.time_range_start) : null,
    time_range_end: form.time_range_enabled ? trimmedOrNull(form.time_range_end) : null,
    is_active: form.is_active,
  };
}

export function apiKeyFormFromKey(apiKey: ProviderApiKey): ApiKeyForm {
  return {
    name: apiKey.name,
    api_key: '',
    api_formats: apiKey.api_formats,
    allowed_model_ids: apiKey.allowed_model_ids,
    note: apiKey.note ?? '',
    internal_priority: String(apiKey.internal_priority),
    rpm_limit: String(apiKey.rpm_limit ?? 0),
    time_range_enabled: apiKey.time_range_enabled,
    time_range_start: apiKey.time_range_start ?? '',
    time_range_end: apiKey.time_range_end ?? '',
    is_active: apiKey.is_active,
  };
}

export function apiKeyUpdatePayload(form: ApiKeyForm): ProviderApiKeyUpdate {
  return {
    name: form.name,
    ...(form.api_key.trim() ? { api_key: form.api_key } : {}),
    api_formats: normalizeSelectedApiFormats(form.api_formats),
    allowed_model_ids: normalizeSelectedIds(form.allowed_model_ids),
    note: trimmedOrNull(form.note),
    internal_priority: requiredNumber(form.internal_priority),
    rpm_limit: optionalNumber(form.rpm_limit),
    time_range_enabled: form.time_range_enabled,
    time_range_start: form.time_range_enabled ? trimmedOrNull(form.time_range_start) : null,
    time_range_end: form.time_range_enabled ? trimmedOrNull(form.time_range_end) : null,
    is_active: form.is_active,
  };
}

export function providerModelPayload(form: ProviderModelForm): ProviderModelBindingCreate {
  return {
    global_model_id: form.global_model_id,
    provider_model_name: form.provider_model_name,
  };
}

export function formatApiFormat(value: string) {
  return value
    .split('_')
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

export function defaultEndpointPath(apiFormat: string) {
  return API_FORMAT_DEFAULT_PATHS[apiFormat] ?? '';
}

export function normalizeBaseUrl(value: string) {
  const trimmed = value.trim();
  if (trimmed.endsWith('://')) return trimmed;
  return trimmed.replace(/\/+$/, '');
}

export function providerTypeLabel(value: ProviderType, t: AdminT) {
  const labels: Record<ProviderType, string> = {
    custom: t('providers.providerTypeCustom'),
  };

  return labels[value];
}

function optionalNumberText(value?: number | null) {
  return value === null || value === undefined ? '' : String(value);
}

function optionalNumber(value: string) {
  const trimmed = value.trim();
  return trimmed ? Number(trimmed) : null;
}

function requiredNumber(value: string) {
  return Number(value.trim() || 0);
}

function trimmedOrNull(value: string) {
  const trimmed = value.trim();
  return trimmed || null;
}

function normalizeSelectedApiFormats(values: string[]) {
  const normalized = values.map((value) => value.trim().toLowerCase());
  return Array.from(new Set(normalized.filter(Boolean)));
}

function normalizeSelectedIds(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
}
