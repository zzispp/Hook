export type {
  UsageRecord,
  RequestRecord,
  RequestRecordDetail,
  RequestRecordStatus,
  RequestCandidateDetail,
  UsageRecordListResponse,
  RequestRecordListResponse,
  ActiveRequestRecordResponse,
} from './request-record';

export type ProviderType = 'custom';

export type ProviderSchedulingMode = 'fixed_order' | 'cache_affinity' | 'load_balance';

export type ProviderCooldownRule = {
  status_code: number;
  failure_count: number;
  cooldown_seconds: number;
};

export type ProviderCooldownPolicy = {
  window_seconds: number;
  rules: ProviderCooldownRule[];
};

export type Provider = {
  id: string;
  name: string;
  provider_type: ProviderType;
  max_retries?: number | null;
  request_timeout_seconds?: number | null;
  stream_first_byte_timeout_seconds?: number | null;
  stream_idle_timeout_seconds?: number | null;
  priority: number;
  keep_priority_on_conversion: boolean;
  enable_format_conversion: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type ProviderListResponse = {
  providers: Provider[];
  total: number;
};

export type ProviderCooldown = {
  provider_id: string;
  provider_name: string;
  status_code: number;
  observed_count: number;
  threshold_count: number;
  window_seconds: number;
  cooldown_seconds: number;
  triggered_at: string;
  cooldown_until: string;
  released_at?: string | null;
  request_id: string;
  candidate_index: number;
  retry_index: number;
  endpoint_id?: string | null;
  endpoint_name?: string | null;
  key_id?: string | null;
  key_name?: string | null;
  error_type?: string | null;
  error_message?: string | null;
  error_code?: string | null;
  error_param?: string | null;
  created_at: string;
  updated_at: string;
};

export type ProviderCooldownListResponse = {
  cooldowns: ProviderCooldown[];
  total: number;
};

export type ProviderCreate = {
  name: string;
  provider_type: ProviderType;
  max_retries?: number | null;
  request_timeout_seconds?: number | null;
  stream_first_byte_timeout_seconds?: number | null;
  stream_idle_timeout_seconds?: number | null;
  priority?: number;
  keep_priority_on_conversion?: boolean;
  enable_format_conversion?: boolean;
  is_active?: boolean;
};

export type ProviderUpdate = Partial<ProviderCreate>;

export type BodyRuleConditionOp =
  | 'eq'
  | 'neq'
  | 'gt'
  | 'lt'
  | 'gte'
  | 'lte'
  | 'starts_with'
  | 'ends_with'
  | 'contains'
  | 'matches'
  | 'exists'
  | 'not_exists'
  | 'in'
  | 'type_is';

export type BodyRuleConditionLeaf = {
  path: string;
  op: BodyRuleConditionOp;
  value?: unknown;
  source?: 'current' | 'original';
};

export type BodyRuleCondition =
  | BodyRuleConditionLeaf
  | { all: BodyRuleCondition[] }
  | { any: BodyRuleCondition[] };

export type HeaderRule =
  | { action: 'set'; key: string; value: string; condition?: BodyRuleCondition }
  | { action: 'drop'; key: string; condition?: BodyRuleCondition }
  | { action: 'rename'; from: string; to: string; condition?: BodyRuleCondition };

export type BodyRuleNameStyle =
  | 'snake_case'
  | 'camelCase'
  | 'PascalCase'
  | 'kebab-case'
  | 'capitalize';

export type BodyRule =
  | { action: 'set'; path: string; value: unknown; condition?: BodyRuleCondition }
  | { action: 'drop'; path: string; condition?: BodyRuleCondition }
  | { action: 'rename'; from: string; to: string; condition?: BodyRuleCondition }
  | { action: 'append'; path: string; value: unknown; condition?: BodyRuleCondition }
  | { action: 'insert'; path: string; index: number; value: unknown; condition?: BodyRuleCondition }
  | {
      action: 'regex_replace';
      path: string;
      pattern: string;
      replacement: string;
      flags?: string;
      count?: number;
      condition?: BodyRuleCondition;
    }
  | { action: 'name_style'; path: string; style: BodyRuleNameStyle; condition?: BodyRuleCondition };

export type ProviderEndpoint = {
  id: string;
  provider_id: string;
  api_format: string;
  base_url: string;
  custom_path?: string | null;
  max_retries?: number | null;
  is_active: boolean;
  format_acceptance_config?: Record<string, unknown> | null;
  header_rules?: HeaderRule[] | null;
  body_rules?: BodyRule[] | null;
  created_at: string;
  updated_at: string;
};

export type ProviderEndpointCreate = {
  api_format: string;
  base_url: string;
  custom_path?: string | null;
  max_retries?: number | null;
  is_active?: boolean;
  format_acceptance_config?: Record<string, unknown> | null;
  header_rules?: HeaderRule[] | null;
  body_rules?: BodyRule[] | null;
};

export type ProviderEndpointUpdate = Partial<Omit<ProviderEndpointCreate, 'api_format'>> & {
  api_format?: string;
};

export type ProviderApiKey = {
  id: string;
  provider_id: string;
  name: string;
  api_formats: string[];
  allowed_model_ids: string[];
  note?: string | null;
  internal_priority: number;
  rpm_limit?: number | null;
  learned_rpm_limit?: number | null;
  cache_ttl_minutes: number;
  max_probe_interval_minutes: number;
  time_range_enabled: boolean;
  time_range_start?: string | null;
  time_range_end?: string | null;
  health_by_format?: Record<string, unknown> | null;
  circuit_breaker_by_format?: Record<string, unknown> | null;
  is_active: boolean;
  has_api_key: boolean;
  created_at: string;
  updated_at: string;
};

export type ProviderApiKeyCreate = {
  name: string;
  api_key: string;
  api_formats: string[];
  allowed_model_ids: string[];
  note?: string | null;
  internal_priority?: number;
  rpm_limit?: number | null;
  cache_ttl_minutes?: number;
  max_probe_interval_minutes?: number;
  time_range_enabled?: boolean;
  time_range_start?: string | null;
  time_range_end?: string | null;
  is_active?: boolean;
};

export type ProviderApiKeyUpdate = Partial<Omit<ProviderApiKeyCreate, 'api_key'>> & {
  api_key?: string;
};

export type ProviderModelReasoningEffort = 'minimal' | 'low' | 'medium' | 'high';

export type ProviderModelMapping = {
  name: string;
  reasoning_effort?: ProviderModelReasoningEffort | null;
};

export type ProviderModelBinding = {
  id: string;
  provider_id: string;
  global_model_id: string;
  provider_model_name: string;
  provider_model_mapping?: ProviderModelMapping | null;
  is_active: boolean;
  config?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
};

export type ProviderModelBindingCreate = {
  global_model_id: string;
  provider_model_name: string;
  provider_model_mapping?: ProviderModelMapping | null;
  config?: Record<string, unknown> | null;
};

export type ProviderModelBindingBatchUpdate = {
  create: ProviderModelBindingCreate[];
  delete_ids: string[];
};

export type ProviderModelBindingUpdate = {
  provider_model_name?: string;
  is_active?: boolean;
  provider_model_mapping?: ProviderModelMapping | null;
  config?: Record<string, unknown> | null;
};

export type ProviderModelCostMode = 'per_request' | 'per_token';

export type ProviderModelCostSource = 'configured' | 'global_default';

export type ProviderModelCost = {
  id: string;
  provider_id: string;
  key_id: string;
  provider_model_id: string;
  cost_mode: ProviderModelCostMode;
  price_per_request?: number | null;
  input_price_per_million?: number | null;
  output_price_per_million?: number | null;
  cache_creation_price_per_million?: number | null;
  cache_read_price_per_million?: number | null;
  created_at: string;
  updated_at: string;
};

export type ProviderModelCostUpsert = {
  provider_model_id: string;
  cost_mode: ProviderModelCostMode;
  price_per_request?: number | null;
  input_price_per_million?: number | null;
  output_price_per_million?: number | null;
  cache_creation_price_per_million?: number | null;
  cache_read_price_per_million?: number | null;
};

export type ProviderModelCostBatchUpsert = {
  costs: ProviderModelCostUpsert[];
};

export type ProviderModelCostListResponse = {
  costs: ProviderModelCost[];
};

export type ProviderUpstreamModelsResponse = {
  models: string[];
};

export type ProviderModelTestRequest = {
  endpoint_id: string;
  request_headers: Record<string, string>;
  request_body: Record<string, unknown>;
};

export type ProviderModelTestEndpoint = {
  id: string;
  api_format: string;
  base_url: string;
};

export type ProviderModelTestResponse = {
  success: boolean;
  model: string;
  endpoint: ProviderModelTestEndpoint;
  status_code?: number | null;
  latency_ms: number;
  request_url: string;
  request_body: unknown;
  response_headers: Record<string, string>;
  response_body: unknown;
  error?: string | null;
};
