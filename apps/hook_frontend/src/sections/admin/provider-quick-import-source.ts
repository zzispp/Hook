import type {
  NewApiQuickImportConfig,
  ProviderQuickImportSourceKind,
  Sub2ApiTokenQuickImportConfig,
  ProviderQuickImportSourceConfig,
  Sub2ApiPasswordQuickImportConfig,
} from 'src/types/provider-quick-import';

export type QuickImportSourceKindState = ProviderQuickImportSourceKind | '';
export type QuickImportAuthTab = 'password' | 'token';

export type QuickImportSourceFields = {
  sourceKind: QuickImportSourceKindState;
  sub2apiAuthTab: QuickImportAuthTab;
  baseUrl: string;
  systemAccessToken: string;
  userId: string;
  email: string;
  password: string;
  authToken: string;
  refreshToken: string;
  tokenExpiresAt: string;
};

export const DEFAULT_QUICK_IMPORT_AUTH_TAB: QuickImportAuthTab = 'password';

export function quickImportSourceReady(form: QuickImportSourceFields) {
  if (form.sourceKind === 'newapi') {
    return Boolean(form.baseUrl.trim() && form.systemAccessToken.trim() && form.userId.trim());
  }
  if (form.sourceKind === 'sub2api') {
    if (form.sub2apiAuthTab === 'password') {
      return Boolean(form.baseUrl.trim() && form.email.trim() && form.password.trim());
    }
    return Boolean(form.baseUrl.trim() && form.authToken.trim() && form.refreshToken.trim() && form.tokenExpiresAt.trim());
  }
  return false;
}

export function quickImportSourceConfig(form: QuickImportSourceFields): ProviderQuickImportSourceConfig {
  if (form.sourceKind === 'newapi') {
    return {
      kind: 'newapi',
      base_url: trimmedBaseUrl(form.baseUrl),
      system_access_token: form.systemAccessToken.trim(),
      user_id: form.userId.trim(),
    } satisfies NewApiQuickImportConfig & { kind: 'newapi' };
  }
  if (form.sourceKind === 'sub2api') {
    if (form.sub2apiAuthTab === 'password') {
      return {
        kind: 'sub2api',
        auth_mode: 'password',
        base_url: trimmedBaseUrl(form.baseUrl),
        email: form.email.trim(),
        password: form.password.trim(),
      } satisfies Sub2ApiPasswordQuickImportConfig & { kind: 'sub2api' };
    }
    return {
      kind: 'sub2api',
      auth_mode: 'token',
      base_url: trimmedBaseUrl(form.baseUrl),
      auth_token: form.authToken.trim(),
      refresh_token: form.refreshToken.trim(),
      token_expires_at: form.tokenExpiresAt.trim(),
    } satisfies Sub2ApiTokenQuickImportConfig & { kind: 'sub2api' };
  }
  throw new Error('source kind is not selected');
}

function trimmedBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, '');
}
