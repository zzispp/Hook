import type { TieredPricingConfig } from './model';

export type ProviderType = 'custom';

export type ProviderSchedulingMode = 'fixed_order' | 'cache_affinity' | 'load_balance';

export type Provider = {
  id: string;
  name: string;
  provider_type: ProviderType;
  max_retries?: number | null;
  request_timeout_seconds?: number | null;
  stream_first_byte_timeout_seconds?: number | null;
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

export type ProviderCreate = {
  name: string;
  provider_type: ProviderType;
  max_retries?: number | null;
  request_timeout_seconds?: number | null;
  stream_first_byte_timeout_seconds?: number | null;
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

export type ProviderModelBinding = {
  id: string;
  provider_id: string;
  global_model_id: string;
  provider_model_name: string;
  is_active: boolean;
  price_per_request?: number | null;
  tiered_pricing?: TieredPricingConfig | null;
  config?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
};

export type ProviderModelBindingCreate = {
  global_model_id: string;
  provider_model_name: string;
  config?: Record<string, unknown> | null;
};

export type ProviderModelBindingUpdate = {
  provider_model_name?: string;
  is_active?: boolean;
  config?: Record<string, unknown> | null;
};

export type RequestRecordStatus = 'pending' | 'streaming' | 'success' | 'failed' | 'cancelled';

export type RequestRecord = {
  request_id: string;
  created_at: string;
  user_id?: string | null;
  username?: string | null;
  token_id?: string | null;
  token_name?: string | null;
  token_prefix?: string | null;
  group_code?: string | null;
  global_model_id?: string | null;
  model_name?: string | null;
  provider_id?: string | null;
  provider_name?: string | null;
  provider_key_name?: string | null;
  provider_key_preview?: string | null;
  client_api_format: string;
  provider_api_format?: string | null;
  request_type: string;
  is_stream: boolean;
  has_failover: boolean;
  has_retry: boolean;
  status: RequestRecordStatus;
  billing_status: string;
  client_status_code?: number | null;
  client_error_type?: string | null;
  client_error_message?: string | null;
  termination_origin?: string | null;
  termination_reason?: string | null;
  stream_end_reason?: string | null;
  prompt_tokens?: number | null;
  completion_tokens?: number | null;
  total_tokens?: number | null;
  cache_creation_input_tokens?: number | null;
  cache_read_input_tokens?: number | null;
  total_cost: number;
  token_cost: number;
  base_cost: number;
  billing_multiplier: number;
  cost_currency: string;
  first_byte_time_ms?: number | null;
  total_latency_ms?: number | null;
  candidate_count: number;
};

export type RequestRecordListResponse = {
  records: RequestRecord[];
  total: number;
};

export type ActiveRequestRecordResponse = {
  records: RequestRecord[];
};

export type RequestCandidateDetail = {
  id: string;
  request_id: string;
  provider_id?: string | null;
  provider_name?: string | null;
  endpoint_id?: string | null;
  endpoint_name?: string | null;
  key_id?: string | null;
  key_name?: string | null;
  key_preview?: string | null;
  client_api_format: string;
  provider_api_format?: string | null;
  needs_conversion: boolean;
  is_stream: boolean;
  candidate_index: number;
  retry_index: number;
  status: string;
  skip_reason?: string | null;
  status_code?: number | null;
  prompt_tokens?: number | null;
  completion_tokens?: number | null;
  total_tokens?: number | null;
  cache_creation_input_tokens?: number | null;
  cache_read_input_tokens?: number | null;
  token_cost?: number | null;
  base_cost?: number | null;
  total_cost?: number | null;
  billing_multiplier?: number | null;
  cost_currency?: string | null;
  latency_ms?: number | null;
  first_byte_time_ms?: number | null;
  error_type?: string | null;
  error_message?: string | null;
  error_code?: string | null;
  error_param?: string | null;
  provider_request_headers?: unknown | null;
  provider_request_body?: unknown | null;
  provider_response_headers?: unknown | null;
  provider_response_body?: unknown | null;
  created_at: string;
  started_at?: string | null;
  finished_at?: string | null;
};

export type RequestRecordDetail = {
  record: RequestRecord;
  candidates: RequestCandidateDetail[];
  request_headers?: unknown | null;
  request_body?: unknown | null;
  client_response_headers?: unknown | null;
  client_response_body?: unknown | null;
};
