use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, Statement, Value};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecord, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse,
};

use crate::{
    StorageError, StorageResult,
    provider::record::{RequestRecordSummaryRecord, request_candidates, request_records},
};

use super::{
    request_record_detail::{candidate_detail, detail_payload, format_timestamp},
    request_record_summary::DEFAULT_COST_CURRENCY,
};

pub async fn list_request_records(store: &super::ProviderStore, request: RequestRecordListRequest) -> StorageResult<RequestRecordListResponse> {
    let total = count_summary_records(store, &request).await?;
    let summaries = list_summary_records(store, &request).await?;
    let records = summaries.into_iter().map(summary_record).collect();
    Ok(RequestRecordListResponse { records, total })
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
            .filter(request_records::Column::Status.is_in(["pending", "streaming"]))
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
    let mut filters = FilterSql::from_request(request);
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
        cost_currency: record.cost_currency.unwrap_or_else(|| DEFAULT_COST_CURRENCY.into()),
        first_byte_time_ms: record.first_byte_time_ms,
        total_latency_ms: record.total_latency_ms,
        candidate_count: record.candidate_count.try_into().unwrap_or(0),
    }
}

struct FilterSql {
    where_clause: String,
    values: Vec<Value>,
}

impl FilterSql {
    fn from_request(request: &RequestRecordListRequest) -> Self {
        let mut params = SqlParams::default();
        let mut filters = Vec::new();
        add_eq_filter(&mut filters, &mut params, "r.status", request.status.as_deref());
        add_eq_filter(&mut filters, &mut params, "r.global_model_id", request.model_id.as_deref());
        add_eq_filter(&mut filters, &mut params, "r.provider_id", request.provider_id.as_deref());
        add_api_format_filter(&mut filters, &mut params, request.api_format.as_deref());
        add_type_filter(&mut filters, request.type_filter.as_deref());
        add_search_filter(&mut filters, &mut params, request.search.as_deref());
        Self {
            where_clause: where_clause(filters),
            values: params.values,
        }
    }

    fn push<T>(&mut self, value: T) -> String
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }
}

#[derive(Default)]
struct SqlParams {
    values: Vec<Value>,
}

impl SqlParams {
    fn push<T>(&mut self, value: T) -> String
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }
}

fn add_eq_filter(filters: &mut Vec<String>, params: &mut SqlParams, column: &str, value: Option<&str>) {
    if let Some(value) = non_empty(value) {
        let placeholder = params.push(value.to_owned());
        filters.push(format!("{column} = {placeholder}"));
    }
}

fn add_api_format_filter(filters: &mut Vec<String>, params: &mut SqlParams, value: Option<&str>) {
    if let Some(value) = non_empty(value) {
        let placeholder = params.push(value.to_owned());
        filters.push(format!("(r.client_api_format = {placeholder} OR r.provider_api_format = {placeholder})"));
    }
}

fn add_type_filter(filters: &mut Vec<String>, value: Option<&str>) {
    match non_empty(value) {
        Some("stream") => filters.push("r.is_stream = TRUE".into()),
        Some("non_stream") => filters.push("r.is_stream = FALSE".into()),
        _ => {}
    }
}

fn add_search_filter(filters: &mut Vec<String>, params: &mut SqlParams, value: Option<&str>) {
    if let Some(value) = non_empty(value) {
        let placeholder = params.push(format!("%{}%", value.to_ascii_lowercase()));
        filters.push(format!("({})", search_conditions(&placeholder).join(" OR ")));
    }
}

fn search_conditions(placeholder: &str) -> Vec<String> {
    [
        "LOWER(r.request_id)",
        "LOWER(COALESCE(r.user_id_snapshot, ''))",
        "LOWER(COALESCE(r.username_snapshot, ''))",
        "LOWER(COALESCE(r.token_name_snapshot, ''))",
        "LOWER(COALESCE(r.token_prefix_snapshot, ''))",
        "LOWER(COALESCE(r.model_name_snapshot, ''))",
        "LOWER(COALESCE(r.provider_name_snapshot, ''))",
        "LOWER(COALESCE(r.provider_key_name_snapshot, ''))",
    ]
    .into_iter()
    .map(|column| format!("{column} LIKE {placeholder}"))
    .collect()
}

fn where_clause(filters: Vec<String>) -> String {
    if filters.is_empty() {
        return String::new();
    }
    format!("WHERE {}", filters.join(" AND "))
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn pagination_value(field: &str, value: u64) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::Database(format!("request record {field} exceeds PostgreSQL integer range")))
}
