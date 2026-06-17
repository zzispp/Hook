import type {
  Provider,
  ProviderApiKey,
  ProviderEndpoint,
  ProviderModelCost,
  ProviderModelBinding,
  ProviderKeyModelMapping,
} from './provider';

export type ProviderQuickImportSourceKind = 'newapi';
export type ProviderQuickImportCostSyncMode = 'overwrite' | 'report_only';
export type ProviderQuickImportUpstreamAnomalyAction = 'disable_key' | 'report_only';
export type ProviderQuickImportGroupChangedAction = 'disable_key' | 'report_only' | 'sync';
export type ProviderQuickImportFetchFailureAction = 'report_only' | 'disable_after_failures';
export type ProviderQuickImportSyncStatus =
  | 'ok'
  | 'sync_disabled'
  | 'source_not_configured'
  | 'source_fetch_failed'
  | 'upstream_token_deleted'
  | 'upstream_token_disabled'
  | 'upstream_group_removed'
  | 'upstream_group_changed'
  | 'upstream_key_unavailable'
  | 'upstream_model_removed'
  | 'no_associated_models'
  | 'cost_unavailable'
  | 'cost_pending_update'
  | 'model_candidate_available';

export type NewApiQuickImportConfig = {
  base_url: string;
  system_access_token: string;
  user_id: string;
};

export type ProviderQuickImportSourceConfig = {
  kind: 'newapi';
} & NewApiQuickImportConfig;

export type ProviderQuickImportPreviewRequest = {
  source_kind: ProviderQuickImportSourceKind;
  source: ProviderQuickImportSourceConfig;
  provider_name: string;
  provider_config: ProviderQuickImportProviderConfig;
  recharge_multiplier: number;
};

export type ProviderQuickImportCommitRequest = ProviderQuickImportPreviewRequest & {
  selected_tokens: ProviderQuickImportSelectedToken[];
  selected_model_ids: string[];
  model_mappings: ProviderQuickImportModelMappingInput[];
  sync_config: ProviderQuickImportSyncConfig;
};

export type ProviderQuickImportAppendPreviewRequest = {
  include_linked_tokens?: boolean;
};

export type ProviderQuickImportAppendCommitRequest = {
  selected_tokens: ProviderQuickImportSelectedToken[];
  selected_model_ids: string[];
  model_mappings: ProviderQuickImportModelMappingInput[];
};

export type ProviderQuickImportBindPreviewRequest = {
  source_kind: ProviderQuickImportSourceKind;
  source: ProviderQuickImportSourceConfig;
  recharge_multiplier: number;
};

export type ProviderQuickImportBindPreviewResponse = {
  provider: Provider;
  local_keys: ProviderQuickImportBindLocalKey[];
  preview: ProviderQuickImportPreviewResponse;
};

export type ProviderQuickImportBindLocalKey = {
  id: string;
  name: string;
  api_formats: string[];
  allowed_model_ids: string[];
  is_active: boolean;
};

export type ProviderQuickImportBindSelectedToken = ProviderQuickImportSelectedToken & {
  local_key_id?: string | null;
};

export type ProviderQuickImportBindCommitRequest = ProviderQuickImportBindPreviewRequest & {
  selected_tokens: ProviderQuickImportBindSelectedToken[];
  selected_model_ids: string[];
  model_mappings: ProviderQuickImportModelMappingInput[];
  sync_config: ProviderQuickImportSyncConfig;
};

export type ProviderQuickImportBindCommitResponse = {
  provider: Provider;
  endpoints: ProviderEndpoint[];
  api_keys: ProviderApiKey[];
  model_bindings: ProviderModelBinding[];
  model_costs: ProviderModelCost[];
  bound_token_count: number;
  created_key_count: number;
  reused_key_count: number;
  deleted_key_count: number;
};

export type ProviderQuickImportRelinkRequest = {
  upstream_token_id: string;
  selected_model_ids: string[];
  model_mappings: ProviderQuickImportModelMappingInput[];
};

export type ProviderQuickImportSyncConfig = {
  auto_sync_enabled: boolean;
  cost_sync_mode: ProviderQuickImportCostSyncMode;
  anomaly_actions: ProviderQuickImportAnomalyActions;
  fetch_failure_action: ProviderQuickImportFetchFailureAction;
  fetch_failure_disable_threshold: number;
};

export type ProviderQuickImportAnomalyActions = {
  token_deleted: ProviderQuickImportUpstreamAnomalyAction;
  token_disabled: ProviderQuickImportUpstreamAnomalyAction;
  group_removed: ProviderQuickImportUpstreamAnomalyAction;
  group_changed: ProviderQuickImportGroupChangedAction;
  key_unavailable: ProviderQuickImportUpstreamAnomalyAction;
  model_removed: ProviderQuickImportUpstreamAnomalyAction;
};

export type ProviderQuickImportKeySyncInfo = {
  source_kind: ProviderQuickImportSourceKind;
  upstream_token_id: string;
  upstream_group?: string | null;
  upstream_group_ratio: number;
  effective_cost_multiplier: number;
  statuses: ProviderQuickImportSyncStatus[];
  last_synced_at?: string | null;
  last_error?: string | null;
};

export type ProviderQuickImportSyncSettingsResponse = {
  provider_id: string;
  source_kind?: ProviderQuickImportSourceKind | null;
  base_url?: string | null;
  user_id?: string | null;
  has_system_access_token: boolean;
  recharge_multiplier?: number | null;
  sync_config: ProviderQuickImportSyncConfig;
  last_status?: ProviderQuickImportSyncStatus | null;
  last_error?: string | null;
  last_synced_at?: string | null;
  consecutive_failures: number;
};

export type ProviderQuickImportSyncSettingsUpdate = {
  base_url?: string;
  user_id?: string;
  system_access_token?: string;
  recharge_multiplier?: number;
  sync_config?: ProviderQuickImportSyncConfig;
};

export type ProviderQuickImportSelectedToken = {
  upstream_token_id: string;
  name: string;
  endpoint_formats: string[];
  effective_cost_multiplier: number;
};

export type ProviderQuickImportProviderConfig = {
  max_retries?: number | null;
  request_timeout_seconds?: number | null;
  stream_first_byte_timeout_seconds?: number | null;
  stream_idle_timeout_seconds?: number | null;
  priority?: number;
  keep_priority_on_conversion?: boolean;
  enable_format_conversion?: boolean;
  is_active?: boolean;
};

export type ProviderQuickImportModelMappingInput = {
  upstream_model_id: string;
  global_model_id: string;
};

export type ProviderQuickImportPreviewResponse = {
  provider_id?: string | null;
  source_kind: ProviderQuickImportSourceKind;
  provider_name: string;
  recharge_multiplier: number;
  tokens: ProviderQuickImportTokenPreview[];
  model_mappings: ProviderQuickImportModelMappingPreview[];
};

export type ProviderQuickImportTokenPreview = {
  upstream_token_id: string;
  name: string;
  masked_key: string;
  status: number;
  group?: string | null;
  group_ratio: number;
  effective_cost_multiplier: number;
  importable: boolean;
  already_imported: boolean;
  import_block_reason?: string | null;
  linked_key?: ProviderQuickImportLinkedKeyPreview | null;
  models: ProviderQuickImportRemoteModel[];
  cost_issues: ProviderQuickImportCostIssue[];
};

export type ProviderQuickImportLinkedKeyPreview = {
  key_id: string;
  name: string;
  endpoint_formats: string[];
  effective_cost_multiplier: number;
  model_mappings: ProviderQuickImportModelMappingInput[];
};

export type ProviderQuickImportRemoteModel = {
  upstream_model_id: string;
  suggested_global_model_id?: string | null;
  supported_endpoint_types: string[];
};

export type ProviderQuickImportModelMappingPreview = {
  upstream_model_id: string;
  suggested_global_model_id?: string | null;
  required: boolean;
};

export type ProviderQuickImportCostIssue = {
  upstream_model_id: string;
  global_model_id?: string | null;
  message: string;
};

export type ProviderQuickImportCommitResponse = {
  provider: Provider;
  endpoints: ProviderEndpoint[];
  api_keys: ProviderApiKey[];
  model_bindings: ProviderModelBinding[];
  model_costs: ProviderModelCost[];
  imported_token_count: number;
  imported_model_count: number;
};

export type ProviderQuickImportResolutionResponse = {
  provider_id: string;
  key_id: string;
  key_name: string;
  source_kind: ProviderQuickImportSourceKind;
  current_upstream_token_id: string;
  current_upstream_group?: string | null;
  current_effective_cost_multiplier: number;
  statuses: ProviderQuickImportSyncStatus[];
  tokens: ProviderQuickImportTokenPreview[];
  model_mappings: ProviderQuickImportModelMappingPreview[];
  associated_models: ProviderKeyModelMapping[];
};
