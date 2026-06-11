import type {
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
  baseUrl: string;
  userId: string;
  systemAccessToken: string;
  rechargeMultiplier: string;
  sync: QuickImportSyncConfigForm;
};

export const DEFAULT_QUICK_IMPORT_SYNC_SETTINGS_FORM: QuickImportSyncSettingsForm = {
  hasSource: false,
  baseUrl: '',
  userId: '',
  systemAccessToken: '',
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
    baseUrl: response.base_url ?? '',
    userId: response.user_id ?? '',
    systemAccessToken: '',
    rechargeMultiplier: String(response.recharge_multiplier ?? 1),
    sync: syncConfigFormFromConfig(response.sync_config),
  };
}

export function syncSettingsPayload(form: QuickImportSyncSettingsForm) {
  const payload: ProviderQuickImportSyncSettingsUpdate = {
    base_url: trimmedBaseUrl(form.baseUrl),
    user_id: form.userId.trim(),
    recharge_multiplier: Number(form.rechargeMultiplier),
    sync_config: syncConfigPayload(form.sync),
  };
  if (form.systemAccessToken.trim()) {
    payload.system_access_token = form.systemAccessToken.trim();
  }
  return payload;
}

export function validSyncSettings(form: QuickImportSyncSettingsForm) {
  return Boolean(
    form.hasSource &&
      form.baseUrl.trim() &&
      form.userId.trim() &&
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
