import type { BillingGroup } from 'src/types/group';
import type { TokenForm } from './api-token-management-types';
import type { ApiToken, ApiTokenCreate, ApiTokenUpdate, AdminApiTokenCreate } from 'src/types/api-token';

export const DEFAULT_SYSTEM_GROUP_CODE = 'default';

export const DEFAULT_TOKEN_FORM: TokenForm = {
  name: '',
  token_type: 'user',
  user_id: '',
  group_code: '',
  expires_at: '',
  model_access_mode: 'all',
  allowed_model_ids: [],
  rate_limit_rpm: '0',
  quota_limit: '',
};

export function userTokenCreatePayload(form: TokenForm): ApiTokenCreate {
  return {
    name: form.name,
    group_code: form.group_code,
    expires_at: localDatetimeToRfc3339(form.expires_at),
    model_access_mode: form.model_access_mode,
    allowed_model_ids: allowedModelIds(form),
    rate_limit_rpm: rateLimitValue(form.rate_limit_rpm),
    quota_limit: optionalNumber(form.quota_limit),
  };
}

export function adminTokenCreatePayload(form: TokenForm): AdminApiTokenCreate {
  return {
    ...userTokenCreatePayload(form),
    token_type: form.token_type,
    user_id: form.token_type === 'user' ? form.user_id.trim() : null,
  };
}

export function tokenUpdatePayload(form: TokenForm): ApiTokenUpdate {
  return {
    name: form.name,
    group_code: form.group_code,
    model_access_mode: form.model_access_mode,
    allowed_model_ids: allowedModelIds(form),
    rate_limit_rpm: rateLimitValue(form.rate_limit_rpm),
    quota_limit: optionalNumber(form.quota_limit),
  };
}

export function formFromToken(token: ApiToken): TokenForm {
  return {
    name: token.name,
    token_type: token.token_type,
    user_id: token.user_id ?? '',
    group_code: token.group_code,
    expires_at: rfc3339ToLocalDatetime(token.expires_at),
    model_access_mode: token.model_access_mode,
    allowed_model_ids: token.allowed_model_ids,
    rate_limit_rpm: String(token.rate_limit_rpm ?? 0),
    quota_limit: token.quota_limit ? String(token.quota_limit) : '',
  };
}

export function defaultCreateForm(defaultGroup: string, defaultUserId = ''): TokenForm {
  return {
    ...DEFAULT_TOKEN_FORM,
    group_code: defaultGroup,
    user_id: defaultUserId,
  };
}

export function defaultGroupCode(groups: Pick<BillingGroup, 'code' | 'is_system'>[]) {
  const systemGroup = groups.find((group) => group.code === DEFAULT_SYSTEM_GROUP_CODE && group.is_system);
  return systemGroup?.code ?? groups.find((group) => group.code === DEFAULT_SYSTEM_GROUP_CODE)?.code ?? '';
}

export function formatCurrency(value: number) {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 });
}

export function formatInteger(value: number) {
  return value.toLocaleString();
}

export function formatTime(value?: string | null) {
  return value ? new Date(value).toLocaleString() : '-';
}

function allowedModelIds(form: TokenForm) {
  return form.model_access_mode === 'limited' ? form.allowed_model_ids : [];
}

function rateLimitValue(value: string) {
  return value.trim() ? Number(value) : 0;
}

function optionalNumber(value: string) {
  return value.trim() ? Number(value) : null;
}

function localDatetimeToRfc3339(value: string) {
  return value ? new Date(value).toISOString() : null;
}

function rfc3339ToLocalDatetime(value?: string | null) {
  return value ? new Date(value).toISOString().slice(0, 16) : '';
}
