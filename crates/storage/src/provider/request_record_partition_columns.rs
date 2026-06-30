pub(super) const REQUEST_RECORD_PARTITION_TABLE: &str = "request_records_partitioned";
pub(super) const REQUEST_CANDIDATE_PARTITION_TABLE: &str = "request_candidates_partitioned";
pub(super) const REQUEST_PAYLOAD_TABLE: &str = "request_payloads";

pub(super) const REQUEST_RECORD_METADATA_COLUMNS: &str = "\
request_id, token_id, user_id_snapshot, username_snapshot, token_name_snapshot, token_prefix_snapshot, group_code, global_model_id, model_name_snapshot, \
provider_id, provider_name_snapshot, endpoint_id, key_id, provider_key_name_snapshot, provider_key_preview_snapshot, client_api_format, provider_api_format, \
request_type, is_stream, has_failover, has_retry, status, billing_status, client_status_code, client_error_type, client_error_message, termination_origin, \
termination_reason, stream_end_reason, prompt_tokens, completion_tokens, total_tokens, cache_creation_input_tokens, cache_read_input_tokens, input_text_tokens, \
input_audio_tokens, input_image_tokens, output_text_tokens, output_audio_tokens, output_image_tokens, reasoning_tokens, cache_creation_5m_input_tokens, \
cache_creation_1h_input_tokens, usage_source, usage_semantic, service_tier, upstream_cost_mode, upstream_cost_source, upstream_price_per_request, \
upstream_input_price_per_million, upstream_output_price_per_million, upstream_cache_creation_price_per_million, upstream_cache_read_price_per_million, \
upstream_request_cost, upstream_input_cost, upstream_output_cost, upstream_cache_creation_cost, upstream_cache_read_cost, upstream_total_cost, input_cost, \
output_cost, cache_creation_cost, cache_read_cost, request_cost, input_price_per_million, output_price_per_million, cache_creation_price_per_million, \
cache_read_price_per_million, cost_currency, token_cost, base_cost, total_cost, billing_multiplier, billing_snapshot, response_headers_time_ms, \
first_sse_event_time_ms, first_token_time_ms, first_byte_time_ms, total_latency_ms, candidate_count, created_at, started_at, finished_at, updated_at";

pub(super) const REQUEST_CANDIDATE_METADATA_COLUMNS: &str = "\
id, request_id, token_id, group_code, global_model_id, provider_id, provider_name_snapshot, endpoint_id, endpoint_name_snapshot, key_id, key_name_snapshot, \
key_preview_snapshot, client_api_format, provider_api_format, needs_conversion, is_stream, is_cached, candidate_index, retry_index, status, skip_reason, \
status_code, prompt_tokens, completion_tokens, total_tokens, cache_creation_input_tokens, cache_read_input_tokens, input_text_tokens, input_audio_tokens, \
input_image_tokens, output_text_tokens, output_audio_tokens, output_image_tokens, reasoning_tokens, cache_creation_5m_input_tokens, cache_creation_1h_input_tokens, \
usage_source, usage_semantic, service_tier, upstream_cost_mode, upstream_cost_source, upstream_price_per_request, upstream_input_price_per_million, \
upstream_output_price_per_million, upstream_cache_creation_price_per_million, upstream_cache_read_price_per_million, upstream_request_cost, upstream_input_cost, \
upstream_output_cost, upstream_cache_creation_cost, upstream_cache_read_cost, upstream_total_cost, input_cost, output_cost, cache_creation_cost, cache_read_cost, \
request_cost, input_price_per_million, output_price_per_million, cache_creation_price_per_million, cache_read_price_per_million, cost_currency, token_cost, \
base_cost, total_cost, billing_multiplier, billing_snapshot, latency_ms, response_headers_time_ms, first_sse_event_time_ms, first_token_time_ms, first_byte_time_ms, \
error_type, error_message, error_code, error_param, created_at, started_at, finished_at";

pub(super) const REQUEST_RECORD_MODEL_COLUMNS_PARTITIONED: &str = "\
r.request_id, r.token_id, r.user_id_snapshot, r.username_snapshot, r.token_name_snapshot, r.token_prefix_snapshot, r.group_code, r.global_model_id, \
r.model_name_snapshot, r.provider_id, r.provider_name_snapshot, r.endpoint_id, r.key_id, r.provider_key_name_snapshot, r.provider_key_preview_snapshot, \
r.client_api_format, r.provider_api_format, r.request_type, r.is_stream, r.has_failover, r.has_retry, r.status, r.billing_status, r.client_status_code, \
r.client_error_type, r.client_error_message, r.termination_origin, r.termination_reason, r.stream_end_reason, r.prompt_tokens, r.completion_tokens, r.total_tokens, \
r.cache_creation_input_tokens, r.cache_read_input_tokens, r.input_text_tokens, r.input_audio_tokens, r.input_image_tokens, r.output_text_tokens, r.output_audio_tokens, \
r.output_image_tokens, r.reasoning_tokens, r.cache_creation_5m_input_tokens, r.cache_creation_1h_input_tokens, r.usage_source, r.usage_semantic, r.service_tier, \
r.upstream_cost_mode, r.upstream_cost_source, r.upstream_price_per_request, r.upstream_input_price_per_million, r.upstream_output_price_per_million, \
r.upstream_cache_creation_price_per_million, r.upstream_cache_read_price_per_million, r.upstream_request_cost, r.upstream_input_cost, r.upstream_output_cost, \
r.upstream_cache_creation_cost, r.upstream_cache_read_cost, r.upstream_total_cost, r.input_cost, r.output_cost, r.cache_creation_cost, r.cache_read_cost, r.request_cost, \
r.input_price_per_million, r.output_price_per_million, r.cache_creation_price_per_million, r.cache_read_price_per_million, r.cost_currency, r.token_cost, r.base_cost, \
r.total_cost, r.billing_multiplier, r.billing_snapshot, r.response_headers_time_ms, r.first_sse_event_time_ms, r.first_token_time_ms, r.first_byte_time_ms, r.total_latency_ms, r.candidate_count, NULL::text AS request_headers, \
NULL::text AS request_body, NULL::text AS client_response_headers, NULL::text AS client_response_body, NULL::timestamptz AS payload_compressed_at, r.created_at, \
r.started_at, r.finished_at, r.updated_at";

pub(super) const REQUEST_RECORD_MODEL_COLUMNS_LEGACY: &str = "\
r.request_id, r.token_id, r.user_id_snapshot, r.username_snapshot, r.token_name_snapshot, r.token_prefix_snapshot, r.group_code, r.global_model_id, \
r.model_name_snapshot, r.provider_id, r.provider_name_snapshot, r.endpoint_id, r.key_id, r.provider_key_name_snapshot, r.provider_key_preview_snapshot, \
r.client_api_format, r.provider_api_format, r.request_type, r.is_stream, r.has_failover, r.has_retry, r.status, r.billing_status, r.client_status_code, \
r.client_error_type, r.client_error_message, r.termination_origin, r.termination_reason, r.stream_end_reason, r.prompt_tokens, r.completion_tokens, r.total_tokens, \
r.cache_creation_input_tokens, r.cache_read_input_tokens, r.input_text_tokens, r.input_audio_tokens, r.input_image_tokens, r.output_text_tokens, r.output_audio_tokens, \
r.output_image_tokens, r.reasoning_tokens, r.cache_creation_5m_input_tokens, r.cache_creation_1h_input_tokens, r.usage_source, r.usage_semantic, r.service_tier, \
r.upstream_cost_mode, r.upstream_cost_source, r.upstream_price_per_request, r.upstream_input_price_per_million, r.upstream_output_price_per_million, \
r.upstream_cache_creation_price_per_million, r.upstream_cache_read_price_per_million, r.upstream_request_cost, r.upstream_input_cost, r.upstream_output_cost, \
r.upstream_cache_creation_cost, r.upstream_cache_read_cost, r.upstream_total_cost, r.input_cost, r.output_cost, r.cache_creation_cost, r.cache_read_cost, r.request_cost, \
r.input_price_per_million, r.output_price_per_million, r.cache_creation_price_per_million, r.cache_read_price_per_million, r.cost_currency, r.token_cost, r.base_cost, \
r.total_cost, r.billing_multiplier, r.billing_snapshot, r.response_headers_time_ms, r.first_sse_event_time_ms, r.first_token_time_ms, r.first_byte_time_ms, r.total_latency_ms, r.candidate_count, r.request_headers, r.request_body, \
r.client_response_headers, r.client_response_body, r.payload_compressed_at, r.created_at, r.started_at, r.finished_at, r.updated_at";

pub(super) const REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED: &str = "\
r.id, r.request_id, r.token_id, r.group_code, r.global_model_id, r.provider_id, r.provider_name_snapshot, r.endpoint_id, r.endpoint_name_snapshot, r.key_id, \
r.key_name_snapshot, r.key_preview_snapshot, r.client_api_format, r.provider_api_format, r.needs_conversion, r.is_stream, r.is_cached, NULL::text AS provider_request_headers, \
NULL::text AS provider_request_body, NULL::text AS provider_response_headers, NULL::text AS provider_response_body, NULL::timestamptz AS payload_compressed_at, \
r.candidate_index, r.retry_index, r.status, r.skip_reason, r.status_code, r.prompt_tokens, r.completion_tokens, r.total_tokens, r.cache_creation_input_tokens, \
r.cache_read_input_tokens, r.input_text_tokens, r.input_audio_tokens, r.input_image_tokens, r.output_text_tokens, r.output_audio_tokens, r.output_image_tokens, \
r.reasoning_tokens, r.cache_creation_5m_input_tokens, r.cache_creation_1h_input_tokens, r.usage_source, r.usage_semantic, r.service_tier, r.upstream_cost_mode, \
r.upstream_cost_source, r.upstream_price_per_request, r.upstream_input_price_per_million, r.upstream_output_price_per_million, r.upstream_cache_creation_price_per_million, \
r.upstream_cache_read_price_per_million, r.upstream_request_cost, r.upstream_input_cost, r.upstream_output_cost, r.upstream_cache_creation_cost, r.upstream_cache_read_cost, \
r.upstream_total_cost, r.input_cost, r.output_cost, r.cache_creation_cost, r.cache_read_cost, r.request_cost, r.input_price_per_million, r.output_price_per_million, \
r.cache_creation_price_per_million, r.cache_read_price_per_million, r.cost_currency, r.token_cost, r.base_cost, r.total_cost, r.billing_multiplier, r.billing_snapshot, \
r.latency_ms, r.response_headers_time_ms, r.first_sse_event_time_ms, r.first_token_time_ms, r.first_byte_time_ms, r.error_type, r.error_message, r.error_code, r.error_param, r.created_at, r.started_at, r.finished_at";

pub(super) const REQUEST_CANDIDATE_MODEL_COLUMNS_LEGACY: &str = "\
r.id, r.request_id, r.token_id, r.group_code, r.global_model_id, r.provider_id, r.provider_name_snapshot, r.endpoint_id, r.endpoint_name_snapshot, r.key_id, \
r.key_name_snapshot, r.key_preview_snapshot, r.client_api_format, r.provider_api_format, r.needs_conversion, r.is_stream, r.is_cached, r.provider_request_headers, \
r.provider_request_body, r.provider_response_headers, r.provider_response_body, r.payload_compressed_at, r.candidate_index, r.retry_index, r.status, r.skip_reason, \
r.status_code, r.prompt_tokens, r.completion_tokens, r.total_tokens, r.cache_creation_input_tokens, r.cache_read_input_tokens, r.input_text_tokens, r.input_audio_tokens, \
r.input_image_tokens, r.output_text_tokens, r.output_audio_tokens, r.output_image_tokens, r.reasoning_tokens, r.cache_creation_5m_input_tokens, r.cache_creation_1h_input_tokens, \
r.usage_source, r.usage_semantic, r.service_tier, r.upstream_cost_mode, r.upstream_cost_source, r.upstream_price_per_request, r.upstream_input_price_per_million, \
r.upstream_output_price_per_million, r.upstream_cache_creation_price_per_million, r.upstream_cache_read_price_per_million, r.upstream_request_cost, r.upstream_input_cost, \
r.upstream_output_cost, r.upstream_cache_creation_cost, r.upstream_cache_read_cost, r.upstream_total_cost, r.input_cost, r.output_cost, r.cache_creation_cost, r.cache_read_cost, \
r.request_cost, r.input_price_per_million, r.output_price_per_million, r.cache_creation_price_per_million, r.cache_read_price_per_million, r.cost_currency, r.token_cost, \
r.base_cost, r.total_cost, r.billing_multiplier, r.billing_snapshot, r.latency_ms, r.response_headers_time_ms, r.first_sse_event_time_ms, r.first_token_time_ms, r.first_byte_time_ms, \
r.error_type, r.error_message, r.error_code, r.error_param, r.created_at, r.started_at, r.finished_at";
