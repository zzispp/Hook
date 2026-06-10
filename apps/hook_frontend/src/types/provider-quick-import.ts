import type {
  Provider,
  ProviderApiKey,
  ProviderEndpoint,
  ProviderModelCost,
  ProviderModelBinding,
} from './provider';

export type ProviderQuickImportSourceKind = 'newapi';

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
};

export type ProviderQuickImportSelectedToken = {
  upstream_token_id: string;
  name: string;
  endpoint_formats: string[];
  effective_cost_multiplier: number;
};

export type ProviderQuickImportProviderConfig = {
  provider_group_id?: string | null;
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
  models: ProviderQuickImportRemoteModel[];
  cost_issues: ProviderQuickImportCostIssue[];
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
