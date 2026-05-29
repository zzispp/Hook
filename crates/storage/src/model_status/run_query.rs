use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use time::format_description::well_known::Rfc3339;
use types::model_status::{ModelStatusRunListRequest, ModelStatusRunListResponse, ModelStatusRunResponse, ModelStatusValue};

use crate::{StorageError, StorageResult};

use super::ModelStatusStore;

const RUN_ROWS_SELECT: &str = "SELECT r.id, r.check_id, c.name AS check_name, c.global_model_id, COALESCE(NULLIF(g.display_name, ''), g.name) AS model_name, c.api_format, c.api_token_id, t.name AS api_token_name, r.status, r.latency_ms, r.status_code, r.message, r.checked_at, r.created_at FROM model_status_check_runs r JOIN model_status_checks c ON c.id = r.check_id JOIN global_models g ON g.id = c.global_model_id JOIN api_tokens t ON t.id = c.api_token_id";

pub(super) async fn list_runs(store: &ModelStatusStore, request: ModelStatusRunListRequest) -> StorageResult<ModelStatusRunListResponse> {
    let total = run_count(store, &request).await?;
    let rows = run_rows(store, &request).await?;
    Ok(ModelStatusRunListResponse {
        items: rows.into_iter().map(run_response).collect::<StorageResult<Vec<_>>>()?,
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

async fn run_count(store: &ModelStatusStore, request: &ModelStatusRunListRequest) -> StorageResult<u64> {
    let filter = RunFilter::from_request(request);
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM model_status_check_runs r JOIN model_status_checks c ON c.id = r.check_id JOIN global_models g ON g.id = c.global_model_id JOIN api_tokens t ON t.id = c.api_token_id {}",
        filter.where_sql()
    );
    CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, filter.values))
        .one(store.connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("model status run count returned no rows".into()))
}

async fn run_rows(store: &ModelStatusStore, request: &ModelStatusRunListRequest) -> StorageResult<Vec<RunRow>> {
    let mut filter = RunFilter::from_request(request);
    let limit = filter.push((request.page_size as i64).into());
    let offset = filter.push(((request.page * request.page_size) as i64).into());
    let sql = format!(
        "{RUN_ROWS_SELECT} {} ORDER BY r.checked_at DESC LIMIT {limit} OFFSET {offset}",
        filter.where_sql()
    );
    RunRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, filter.values))
        .all(store.connection())
        .await
        .map_err(Into::into)
}

struct RunFilter {
    parts: Vec<String>,
    values: Vec<Value>,
}

impl RunFilter {
    fn from_request(request: &ModelStatusRunListRequest) -> Self {
        let mut filter = Self {
            parts: Vec::new(),
            values: Vec::new(),
        };
        filter.add_search(request.search.as_deref());
        filter.add_api_format(request.api_format.as_deref());
        filter.add_status(request.status);
        filter
    }

    fn add_search(&mut self, search: Option<&str>) {
        let Some(value) = normalized(search) else {
            return;
        };
        let placeholder = self.push(format!("%{value}%").into());
        self.parts.push(format!(
            "(c.name ILIKE {placeholder} OR g.name ILIKE {placeholder} OR g.display_name ILIKE {placeholder} OR c.api_format ILIKE {placeholder})"
        ));
    }

    fn add_api_format(&mut self, api_format: Option<&str>) {
        let Some(value) = normalized(api_format) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("c.api_format = {placeholder}"));
    }

    fn add_status(&mut self, status: Option<ModelStatusValue>) {
        let Some(value) = status else {
            return;
        };
        let placeholder = self.push(value.as_str().to_owned().into());
        self.parts.push(format!("r.status = {placeholder}"));
    }

    fn push(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("${}", self.values.len())
    }

    fn where_sql(&self) -> String {
        if self.parts.is_empty() {
            return String::new();
        }
        format!("WHERE {}", self.parts.join(" AND "))
    }
}

#[derive(Debug, FromQueryResult)]
struct CountRow {
    total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct RunRow {
    id: String,
    check_id: String,
    check_name: String,
    global_model_id: String,
    model_name: String,
    api_format: String,
    api_token_id: String,
    api_token_name: String,
    status: String,
    latency_ms: Option<i64>,
    status_code: Option<i32>,
    message: Option<String>,
    checked_at: time::OffsetDateTime,
    created_at: time::OffsetDateTime,
}

fn run_response(row: RunRow) -> StorageResult<ModelStatusRunResponse> {
    Ok(ModelStatusRunResponse {
        id: row.id,
        check_id: row.check_id,
        check_name: row.check_name,
        global_model_id: row.global_model_id,
        model_name: row.model_name,
        api_format: row.api_format,
        api_token_id: row.api_token_id,
        api_token_name: row.api_token_name,
        status: status_value(&row.status)?,
        latency_ms: row.latency_ms,
        status_code: row.status_code,
        message: row.message,
        checked_at: format_timestamp(row.checked_at),
        created_at: format_timestamp(row.created_at),
    })
}

fn status_value(value: &str) -> StorageResult<ModelStatusValue> {
    match value {
        "operational" => Ok(ModelStatusValue::Operational),
        "degraded" => Ok(ModelStatusValue::Degraded),
        "failed" => Ok(ModelStatusValue::Failed),
        "error" => Ok(ModelStatusValue::Error),
        other => Err(StorageError::Database(format!("invalid model status value: {other}"))),
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("model status run timestamp must format as RFC3339")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_filter_parameterizes_search_status_and_format() {
        let request = ModelStatusRunListRequest {
            search: Some("gpt%' OR 1=1 --".into()),
            api_format: Some("openai:chat".into()),
            status: Some(ModelStatusValue::Error),
            ..Default::default()
        };

        let filter = RunFilter::from_request(&request);
        let where_sql = filter.where_sql();

        assert!(where_sql.contains("c.name ILIKE $1"));
        assert!(where_sql.contains("c.api_format = $2"));
        assert!(where_sql.contains("r.status = $3"));
        assert!(!where_sql.contains("OR 1=1"));
        assert_eq!(filter.values.len(), 3);
    }
}
