export type RequestRecordStatus = 'pending' | 'streaming' | 'success' | 'failed' | 'cancelled';

type RequestBillingFields = {
  service_tier?: string | null;
  input_cost?: number | null;
  output_cost?: number | null;
  cache_creation_cost?: number | null;
  cache_read_cost?: number | null;
  request_cost?: number | null;
  input_price_per_million?: number | null;
  output_price_per_million?: number | null;
  cache_creation_price_per_million?: number | null;
  cache_read_price_per_million?: number | null;
};

export type RequestRecord = RequestBillingFields & {
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
  input_text_tokens?: number | null;
  input_audio_tokens?: number | null;
  input_image_tokens?: number | null;
  output_text_tokens?: number | null;
  output_audio_tokens?: number | null;
  output_image_tokens?: number | null;
  reasoning_tokens?: number | null;
  cache_creation_5m_input_tokens?: number | null;
  cache_creation_1h_input_tokens?: number | null;
  usage_source?: string | null;
  usage_semantic?: string | null;
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

export type UsageRecord = {
  created_at: string;
  token_name?: string | null;
  token_prefix?: string | null;
  model_name?: string | null;
  client_api_format: string;
  request_type: string;
  is_stream: boolean;
  status: RequestRecordStatus;
  prompt_tokens?: number | null;
  completion_tokens?: number | null;
  total_tokens?: number | null;
  cache_creation_input_tokens?: number | null;
  cache_read_input_tokens?: number | null;
  total_cost: number;
  cost_currency: string;
  first_byte_time_ms?: number | null;
  total_latency_ms?: number | null;
};

export type UsageRecordListResponse = {
  records: UsageRecord[];
  total: number;
};

export type ActiveRequestRecordResponse = {
  records: RequestRecord[];
};

export type RequestCandidateDetail = RequestBillingFields & {
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
  input_text_tokens?: number | null;
  input_audio_tokens?: number | null;
  input_image_tokens?: number | null;
  output_text_tokens?: number | null;
  output_audio_tokens?: number | null;
  output_image_tokens?: number | null;
  reasoning_tokens?: number | null;
  cache_creation_5m_input_tokens?: number | null;
  cache_creation_1h_input_tokens?: number | null;
  usage_source?: string | null;
  usage_semantic?: string | null;
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
