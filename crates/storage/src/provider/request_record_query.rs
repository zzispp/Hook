use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecordDetail, RequestRecordListRequest, RequestRecordListResponse, UsageRecordListResponse,
};

use crate::{
    StorageError, StorageResult,
    provider::record::{RequestCandidateRecord, RequestRecordSummaryRecord, request_candidates, request_records},
};

use super::{
    request_record_filter::{FilterSql, pagination_value},
    request_record_partition_columns::{
        REQUEST_CANDIDATE_MODEL_COLUMNS_LEGACY, REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED, REQUEST_RECORD_MODEL_COLUMNS_LEGACY,
        REQUEST_RECORD_MODEL_COLUMNS_PARTITIONED,
    },
    request_record_query_mapper::{summary_record, usage_record},
    request_record_query_payloads::{candidate_details_with_payloads, record_payloads},
};

const STATUS_PENDING: &str = "pending";
const STATUS_STREAMING: &str = "streaming";

pub async fn list_request_records(store: &super::ProviderStore, request: RequestRecordListRequest) -> StorageResult<RequestRecordListResponse> {
    let total = count_summary_records(store, &request).await?;
    let summaries = list_summary_records(store, &request).await?;
    let records = summaries.into_iter().map(summary_record).collect::<StorageResult<Vec<_>>>()?;
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
    let records = summaries.into_iter().map(summary_record).collect::<StorageResult<Vec<_>>>()?;
    Ok(ActiveRequestRecordResponse { records })
}

pub async fn get_request_record(store: &super::ProviderStore, request_id: &str) -> StorageResult<RequestRecordDetail> {
    let summary = detail_summary_record(store, request_id).await?.ok_or(StorageError::NotFound)?;
    let candidates = detail_candidate_records(store, request_id).await?;
    let record = summary_record(summary.clone())?;
    let record_payloads = record_payloads(store, &summary).await?;
    let details = candidate_details_with_payloads(store, candidates).await?;
    Ok(RequestRecordDetail {
        record,
        candidates: details,
        payloads: record_payloads.payloads,
        request_headers: record_payloads.request_headers,
        request_body: record_payloads.request_body,
        client_response_headers: record_payloads.client_response_headers,
        client_response_body: record_payloads.client_response_body,
    })
}

async fn active_summary_records(store: &super::ProviderStore, ids: &[String]) -> StorageResult<Vec<RequestRecordSummaryRecord>> {
    let mut values = Vec::new();
    let where_clause = active_where_clause(ids, &mut values);
    let sql = format!(
        "SELECT r.* FROM ({}) r {where_clause} ORDER BY r.created_at DESC, r.request_id DESC",
        request_record_summary_union_sql()
    );
    request_records::Model::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
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
    let sql = format!(
        "SELECT COUNT(*) AS total FROM ({}) r {}",
        request_record_summary_union_sql(),
        filters.where_clause
    );
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
        "SELECT r.* FROM ({}) r {} ORDER BY r.created_at DESC, r.request_id DESC LIMIT {limit} OFFSET {offset}",
        request_record_summary_union_sql(),
        filters.where_clause
    );
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, sql, filters.values);
    request_records::Model::find_by_statement(statement)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn detail_summary_record(store: &super::ProviderStore, request_id: &str) -> StorageResult<Option<RequestRecordSummaryRecord>> {
    let sql = format!("SELECT r.* FROM ({}) r WHERE r.request_id = $1 LIMIT 1", request_record_union_sql());
    request_records::Model::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, [Value::from(request_id.to_owned())]))
        .one(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn detail_candidate_records(store: &super::ProviderStore, request_id: &str) -> StorageResult<Vec<RequestCandidateRecord>> {
    let sql = format!(
        "SELECT r.* FROM ({}) r WHERE r.request_id = $1 ORDER BY r.candidate_index ASC, r.retry_index ASC",
        request_candidate_union_sql()
    );
    request_candidates::Model::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, [Value::from(request_id.to_owned())]))
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

fn request_record_union_sql() -> String {
    format!(
        "SELECT {} FROM request_records_partitioned r UNION ALL SELECT {} FROM request_records r WHERE NOT EXISTS \
         (SELECT 1 FROM request_records_partitioned p WHERE p.request_id = r.request_id)",
        REQUEST_RECORD_MODEL_COLUMNS_PARTITIONED, REQUEST_RECORD_MODEL_COLUMNS_LEGACY
    )
}

fn request_record_summary_union_sql() -> String {
    format!(
        "SELECT {} FROM request_records_partitioned r UNION ALL SELECT {} FROM request_records r WHERE NOT EXISTS \
         (SELECT 1 FROM request_records_partitioned p WHERE p.request_id = r.request_id)",
        REQUEST_RECORD_MODEL_COLUMNS_PARTITIONED, REQUEST_RECORD_MODEL_COLUMNS_PARTITIONED
    )
}

fn request_candidate_union_sql() -> String {
    format!(
        "SELECT {} FROM request_candidates_partitioned r UNION ALL SELECT {} FROM request_candidates r WHERE NOT EXISTS \
         (SELECT 1 FROM request_candidates_partitioned p WHERE p.id = r.id)",
        REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED, REQUEST_CANDIDATE_MODEL_COLUMNS_LEGACY
    )
}

fn active_where_clause(ids: &[String], values: &mut Vec<Value>) -> String {
    if ids.is_empty() {
        let pending = push_value(values, STATUS_PENDING);
        let streaming = push_value(values, STATUS_STREAMING);
        return format!("WHERE r.status IN ({pending}, {streaming})");
    }
    let placeholders = ids.iter().map(|id| push_value(values, id.as_str())).collect::<Vec<_>>().join(", ");
    format!("WHERE r.request_id IN ({placeholders})")
}

fn push_value(values: &mut Vec<Value>, value: &str) -> String {
    values.push(Value::from(value.to_owned()));
    format!("${}", values.len())
}
