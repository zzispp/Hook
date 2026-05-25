use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Statement};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecord, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
    UsageRecord, UsageRecordListResponse,
};

use crate::{
    StorageError, StorageResult,
    provider::record::{RequestRecordSummaryRecord, request_candidates, request_records},
};

use super::{
    request_record_detail::{candidate_detail, detail_payload, format_timestamp},
    request_record_filter::{FilterSql, pagination_value},
    request_record_summary::DEFAULT_COST_CURRENCY,
};

const STATUS_PENDING: &str = "pending";
const STATUS_STREAMING: &str = "streaming";
const ACTIVE_REQUEST_STATUSES: [&str; 2] = [STATUS_PENDING, STATUS_STREAMING];

pub async fn list_request_records(store: &super::ProviderStore, request: RequestRecordListRequest) -> StorageResult<RequestRecordListResponse> {
    let total = count_summary_records(store, &request).await?;
    let summaries = list_summary_records(store, &request).await?;
    let records = summaries.into_iter().map(summary_record).collect();
    Ok(RequestRecordListResponse { records, total })
}

pub async fn list_usage_records(store: &super::ProviderStore, user_id: &str, request: RequestRecordListRequest) -> StorageResult<UsageRecordListResponse> {
    let total = count_user_summary_records(store, user_id, &request).await?;
    let summaries = list_user_summary_records(store, user_id, &request).await?;
    let records = summaries.into_iter().map(usage_record).collect();
    Ok(UsageRecordListResponse { records, total })
}

pub async fn list_active_request_records(store: &super::ProviderStore, request: ActiveRequestRecordRequest) -> StorageResult<ActiveRequestRecordResponse> {
    let summaries = active_summary_records(store, &request.ids).await?;
    let records = summaries.into_iter().map(summary_record).collect();
    Ok(ActiveRequestRecordResponse { records })
}

pub async fn get_request_record(store: &super::ProviderStore, request_id: &str) -> StorageResult<RequestRecordDetail> {
    let summary = request_records::Entity::find_by_id(request_id.to_owned())
        .one(store.connection())
        .await?
        .ok_or(StorageError::NotFound)?;
    let candidates = request_candidates::Entity::find()
        .filter(request_candidates::Column::RequestId.eq(request_id))
        .order_by_asc(request_candidates::Column::CandidateIndex)
        .order_by_asc(request_candidates::Column::RetryIndex)
        .all(store.connection())
        .await?;
    let record = summary_record(summary.clone());
    let request_headers = detail_payload(summary.request_headers)?;
    let request_body = detail_payload(summary.request_body)?;
    let client_response_headers = detail_payload(summary.client_response_headers)?;
    let client_response_body = detail_payload(summary.client_response_body)?;
    let details = candidates.into_iter().map(candidate_detail).collect::<StorageResult<Vec<_>>>()?;
    Ok(RequestRecordDetail {
        record,
        candidates: details,
        request_headers,
        request_body,
        client_response_headers,
        client_response_body,
    })
}

async fn active_summary_records(store: &super::ProviderStore, ids: &[String]) -> StorageResult<Vec<RequestRecordSummaryRecord>> {
    if ids.is_empty() {
        return request_records::Entity::find()
            .filter(request_records::Column::Status.is_in(ACTIVE_REQUEST_STATUSES))
            .order_by_desc(request_records::Column::CreatedAt)
            .all(store.connection())
            .await
            .map_err(StorageError::from);
    }
    request_records::Entity::find()
        .filter(request_records::Column::RequestId.is_in(ids.iter().cloned()))
        .order_by_desc(request_records::Column::CreatedAt)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn count_summary_records(store: &super::ProviderStore, request: &RequestRecordListRequest) -> StorageResult<u64> {
    let filters = FilterSql::from_request(request);
    count_summary_records_with_filters(store, filters).await
}

async fn count_user_summary_records(store: &super::ProviderStore, user_id: &str, request: &RequestRecordListRequest) -> StorageResult<u64> {
    let filters = FilterSql::from_user_request(user_id, request);
    count_summary_records_with_filters(store, filters).await
}

async fn count_summary_records_with_filters(store: &super::ProviderStore, filters: FilterSql) -> StorageResult<u64> {
    let sql = format!("SELECT COUNT(*) AS total FROM request_records r {}", filters.where_clause);
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, filters.values);
    let row = store
        .connection()
        .query_one_raw(statement)
        .await?
        .ok_or_else(|| StorageError::Database("request record count query returned no rows".into()))?;
    let total = row.try_get::<i64>("", "total")?;
    Ok(total.try_into().unwrap_or(0))
}

async fn list_summary_records(store: &super::ProviderStore, request: &RequestRecordListRequest) -> StorageResult<Vec<RequestRecordSummaryRecord>> {
    list_summary_records_with_filters(store, FilterSql::from_request(request), request).await
}

async fn list_user_summary_records(
    store: &super::ProviderStore,
    user_id: &str,
    request: &RequestRecordListRequest,
) -> StorageResult<Vec<RequestRecordSummaryRecord>> {
    list_summary_records_with_filters(store, FilterSql::from_user_request(user_id, request), request).await
}

async fn list_summary_records_with_filters(
    store: &super::ProviderStore,
    mut filters: FilterSql,
    request: &RequestRecordListRequest,
) -> StorageResult<Vec<RequestRecordSummaryRecord>> {
    let limit = filters.push(pagination_value("limit", request.limit)?);
    let offset = filters.push(pagination_value("skip", request.skip)?);
    let sql = format!(
        "SELECT r.* FROM request_records r {} ORDER BY r.created_at DESC, r.request_id DESC LIMIT {limit} OFFSET {offset}",
        filters.where_clause
    );
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, filters.values);
    request_records::Model::find_by_statement(statement)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

fn usage_record(record: RequestRecordSummaryRecord) -> UsageRecord {
    UsageRecord {
        created_at: format_timestamp(record.created_at),
        token_name: record.token_name_snapshot,
        token_prefix: record.token_prefix_snapshot,
        model_name: record.model_name_snapshot.or(record.global_model_id),
        client_api_format: record.client_api_format,
        request_type: record.request_type,
        is_stream: record.is_stream,
        prompt_tokens: record.prompt_tokens,
        completion_tokens: record.completion_tokens,
        total_tokens: record.total_tokens,
        cache_creation_input_tokens: record.cache_creation_input_tokens,
        cache_read_input_tokens: record.cache_read_input_tokens,
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        cost_currency: record.cost_currency.unwrap_or_else(|| DEFAULT_COST_CURRENCY.into()),
        first_byte_time_ms: record.first_byte_time_ms,
        total_latency_ms: record.total_latency_ms,
    }
}

fn summary_record(record: RequestRecordSummaryRecord) -> RequestRecord {
    RequestRecord {
        request_id: record.request_id,
        created_at: format_timestamp(record.created_at),
        user_id: record.user_id_snapshot,
        username: record.username_snapshot,
        token_id: record.token_id,
        token_name: record.token_name_snapshot,
        token_prefix: record.token_prefix_snapshot,
        group_code: record.group_code,
        global_model_id: record.global_model_id.clone(),
        model_name: record.model_name_snapshot.or(record.global_model_id),
        provider_id: record.provider_id,
        provider_name: record.provider_name_snapshot,
        provider_key_name: record.provider_key_name_snapshot,
        provider_key_preview: record.provider_key_preview_snapshot,
        client_api_format: record.client_api_format,
        provider_api_format: record.provider_api_format,
        request_type: record.request_type,
        is_stream: record.is_stream,
        has_failover: record.has_failover,
        has_retry: record.has_retry,
        status: record.status,
        billing_status: record.billing_status,
        client_status_code: record.client_status_code,
        client_error_type: record.client_error_type,
        client_error_message: record.client_error_message,
        termination_origin: record.termination_origin,
        termination_reason: record.termination_reason,
        stream_end_reason: record.stream_end_reason,
        prompt_tokens: record.prompt_tokens,
        completion_tokens: record.completion_tokens,
        total_tokens: record.total_tokens,
        cache_creation_input_tokens: record.cache_creation_input_tokens,
        cache_read_input_tokens: record.cache_read_input_tokens,
        input_text_tokens: record.input_text_tokens,
        input_audio_tokens: record.input_audio_tokens,
        input_image_tokens: record.input_image_tokens,
        output_text_tokens: record.output_text_tokens,
        output_audio_tokens: record.output_audio_tokens,
        output_image_tokens: record.output_image_tokens,
        reasoning_tokens: record.reasoning_tokens,
        cache_creation_5m_input_tokens: record.cache_creation_5m_input_tokens,
        cache_creation_1h_input_tokens: record.cache_creation_1h_input_tokens,
        usage_source: record.usage_source,
        usage_semantic: record.usage_semantic,
        service_tier: record.service_tier,
        input_cost: record.input_cost,
        output_cost: record.output_cost,
        cache_creation_cost: record.cache_creation_cost,
        cache_read_cost: record.cache_read_cost,
        request_cost: record.request_cost,
        input_price_per_million: record.input_price_per_million,
        output_price_per_million: record.output_price_per_million,
        cache_creation_price_per_million: record.cache_creation_price_per_million,
        cache_read_price_per_million: record.cache_read_price_per_million,
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        token_cost: record.token_cost.unwrap_or(Decimal::ZERO),
        base_cost: record.base_cost.unwrap_or(Decimal::ZERO),
        billing_multiplier: record.billing_multiplier.unwrap_or(Decimal::ONE),
        billing_snapshot: crate::json::decode_optional(record.billing_snapshot).ok().flatten(),
        cost_currency: record.cost_currency.unwrap_or_else(|| DEFAULT_COST_CURRENCY.into()),
        first_byte_time_ms: record.first_byte_time_ms,
        total_latency_ms: record.total_latency_ms,
        candidate_count: record.candidate_count.try_into().unwrap_or(0),
    }
}
