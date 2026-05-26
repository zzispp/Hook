import type { AdminT } from './shared';
import type { Role, UserInput, SystemUser, UserQuotaMode } from 'src/types/rbac';

import { fNumber } from 'src/utils/format-number';

import { formatWalletMoney } from '../wallet/wallet-display';

export type UserForm = {
  username: string;
  password: string;
  email: string;
  role: string;
  group_code: string;
  is_active: boolean;
  allowed_model_ids: string[];
  allowed_provider_ids: string[];
  rate_limit_rpm: string;
  quota_mode: UserQuotaMode;
};

export const DEFAULT_USER_FORM: UserForm = {
  username: '',
  password: '',
  email: '',
  role: '',
  group_code: 'default',
  is_active: true,
  allowed_model_ids: [],
  allowed_provider_ids: [],
  rate_limit_rpm: '',
  quota_mode: 'wallet',
};

export function enabledRoleOptions(roles: Role[]) {
  return roles.filter((role) => role.enabled);
}

export function roleFilterOptions(roles: Role[]) {
  return roles.map((role) => ({
    value: role.code,
    label: `${displayRole(role.code, roles)} (${role.code})`,
  }));
}

export function formFromUser(user: SystemUser): UserForm {
  return {
    username: user.username,
    password: '',
    email: user.email,
    role: user.role,
    group_code: user.group_code,
    is_active: user.is_active,
    allowed_model_ids: user.allowed_model_ids,
    allowed_provider_ids: user.allowed_provider_ids,
    rate_limit_rpm: user.rate_limit_rpm ? String(user.rate_limit_rpm) : '',
    quota_mode: user.quota_mode,
  };
}

export function formToPayload(form: UserForm): UserInput {
  return {
    username: form.username,
    password: form.password,
    email: form.email,
    role: form.role,
    group_code: form.group_code,
    is_active: form.is_active,
    allowed_model_ids: form.allowed_model_ids,
    allowed_provider_ids: form.allowed_provider_ids,
    rate_limit_rpm: rateLimitValue(form.rate_limit_rpm),
    quota_mode: form.quota_mode,
  };
}

export function displayRole(code: string, roles: Role[]) {
  const role = roles.find((item) => item.code === code);
  return role?.name ?? code;
}

export function walletBalanceText(user: SystemUser, t: AdminT) {
  return user.quota_mode === 'unlimited'
    ? t('users.unlimited')
    : formatUserMoney(user.wallet?.available_balance);
}

export function walletConsumedText(user: SystemUser) {
  return formatUserMoney(user.wallet?.total_consumed);
}

export function userRateLimitText(user: SystemUser, t: AdminT) {
  const limit = user.rate_limit_rpm ?? 0;
  return limit <= 0
    ? t('users.followSystem')
    : `${fNumber(limit, { maximumFractionDigits: 0 })} ${t('tokens.rpm')}`;
}

export function formatUserDateTime(value?: string | null) {
  if (!value) return '-';
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function formatUserMoney(value?: number | null) {
  return formatWalletMoney(value);
}

function rateLimitValue(value: string) {
  const normalized = value.trim();
  if (!normalized || normalized === '0') return null;

  const limit = Number(normalized);
  if (!Number.isFinite(limit)) {
    throw new Error('rate_limit_rpm must be a number');
  }
  return limit;
}
