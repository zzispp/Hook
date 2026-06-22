import type { QuickImportAuthTab } from './provider-quick-import-source';
import type {
  ProviderQuickImportSourceKind,
  ProviderQuickImportSyncConfig,
  ProviderQuickImportSyncSettingsUpdate,
  ProviderQuickImportSyncSettingsResponse,
} from 'src/types/provider-quick-import';

export type QuickImportSyncConfigForm = {
  auto_sync_enabled: boolean;
  cost_sync_mode: ProviderQuickImportSyncConfig['cost_sync_mode'];
  anomaly_actions: ProviderQuickImportSyncConfig['anomaly_actions'];
  fetch_failure_action: ProviderQuickImportSyncConfig['fetch_failure_action'];
  fetch_failure_disable_threshold: string;
};

export type QuickImportSyncSettingsForm = {
  hasSource: boolean;
  sourceKind: ProviderQuickImportSourceKind | '';
  sub2apiAuthTab: QuickImportAuthTab;
  baseUrl: string;
  userId: string;
  email: string;
  password: string;
  authToken: string;
  refreshToken: string;
  tokenExpiresAt: string;
  systemAccessToken: string;
  hasSystemAccessToken: boolean;
  hasPassword: boolean;
  hasAuthToken: boolean;
  hasRefreshToken: boolean;
  rechargeMultiplier: string;
  sync: QuickImportSyncConfigForm;
};

export const DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM: QuickImportSyncSettingsForm = {
  hasSource: false,
  sourceKind: '',
  sub2apiAuthTab: 'password',
  baseUrl: '',
  userId: '',
  email: '',
  password: '',
  authToken: '',
  refreshToken: '',
  tokenExpiresAt: '',
  systemAccessToken: '',
  hasSystemAccessToken: false,
  hasPassword: false,
  hasAuthToken: false,
  hasRefreshToken: false,
  rechargeMultiplier: '1',
  sync: defaultQuickImportSyncConfigForm(),
};

export function syncConfigPayload(form: QuickImportSyncConfigForm): ProviderQuickImportSyncConfig {
  return {
    auto_sync_enabled: form.auto_sync_enabled,
    cost_sync_mode: form.cost_sync_mode,
    anomaly_actions: form.anomaly_actions,
    fetch_failure_action: form.fetch_failure_action,
    fetch_failure_disable_threshold: Number(form.fetch_failure_disable_threshold),
  };
}

export function syncSettingsFormFromResponse(
  response: ProviderQuickImportSyncSettingsResponse
): QuickImportSyncSettingsForm {
  return {
    hasSource: Boolean(response.source_kind),
    sourceKind: response.source_kind ?? '',
    sub2apiAuthTab: response.email ? 'password' : 'token',
    baseUrl: response.base_url ?? '',
    userId: response.user_id ?? '',
    email: response.email ?? '',
    password: '',
    authToken: '',
    refreshToken: '',
    tokenExpiresAt: response.token_expires_at ?? '',
    systemAccessToken: '',
    hasSystemAccessToken: response.has_system_access_token,
    hasPassword: response.has_password ?? false,
    hasAuthToken: response.has_auth_token,
    hasRefreshToken: response.has_refresh_token,
    rechargeMultiplier: String(response.recharge_multiplier ?? 1),
    sync: syncConfigFormFromConfig(response.sync_config),
  };
}

export function syncSettingsPayload(form: QuickImportSyncSettingsForm) {
  const payload: ProviderQuickImportSyncSettingsUpdate = {
    base_url: trimmedBaseUrl(form.baseUrl),
    recharge_multiplier: Number(form.rechargeMultiplier),
    sync_config: syncConfigPayload(form.sync),
  };
  if (form.sourceKind === 'newapi') {
    payload.user_id = form.userId.trim();
    if (form.systemAccessToken.trim()) {
      payload.system_access_token = form.systemAccessToken.trim();
    }
  }
  if (form.sourceKind === 'sub2api') {
    if (form.sub2apiAuthTab === 'password') {
      payload.email = form.email.trim();
      if (form.password.trim()) {
        payload.password = form.password.trim();
      }
    } else {
      if (form.authToken.trim()) {
        payload.auth_token = form.authToken.trim();
      }
      if (form.refreshToken.trim()) {
        payload.refresh_token = form.refreshToken.trim();
      }
      if (form.tokenExpiresAt.trim()) {
        payload.token_expires_at = form.tokenExpiresAt.trim();
      }
    }
  }
  return payload;
}

export function validSyncSettings(form: QuickImportSyncSettingsForm) {
  const sourceReady =
    form.sourceKind === 'newapi'
      ? Boolean(form.baseUrl.trim() && form.userId.trim())
      : form.sourceKind === 'sub2api'
        ? form.sub2apiAuthTab === 'password'
          ? Boolean(form.baseUrl.trim() && form.email.trim())
          : Boolean(form.baseUrl.trim())
        : false;
  return Boolean(
    form.hasSource &&
      sourceReady &&
      Number(form.rechargeMultiplier) > 0 &&
      Number(form.sync.fetch_failure_disable_threshold) > 0
  );
}

export function validSyncConfig(form: QuickImportSyncConfigForm) {
  return Number(form.fetch_failure_disable_threshold) > 0;
}

export function defaultQuickImportSyncConfigForm(): QuickImportSyncConfigForm {
  return {
    auto_sync_enabled: true,
    cost_sync_mode: 'overwrite',
    anomaly_actions: {
      token_deleted: 'disable_key',
      token_disabled: 'disable_key',
      group_removed: 'disable_key',
      group_changed: 'disable_key',
      key_unavailable: 'report_only',
      model_removed: 'disable_key',
    },
    fetch_failure_action: 'report_only',
    fetch_failure_disable_threshold: '3',
  };
}

function syncConfigFormFromConfig(config: ProviderQuickImportSyncConfig): QuickImportSyncConfigForm {
  return {
    auto_sync_enabled: config.auto_sync_enabled,
    cost_sync_mode: config.cost_sync_mode,
    anomaly_actions: config.anomaly_actions,
    fetch_failure_action: config.fetch_failure_action,
    fetch_failure_disable_threshold: String(config.fetch_failure_disable_threshold),
  };
}

function trimmedBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, '');
}
