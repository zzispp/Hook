use sea_orm::Value;
use types::provider::RequestRecordListRequest;

use crate::{StorageError, StorageResult};

const STATUS_FILTER_ACTIVE: &str = "active";
const STATUS_FILTER_FAILOVER: &str = "failover";
const STATUS_FILTER_RETRY: &str = "retry";
const STATUS_PENDING: &str = "pending";
const STATUS_STREAMING: &str = "streaming";

pub(super) struct FilterSql {
    pub(super) where_clause: String,
    pub(super) values: Vec<Value>,
}

impl FilterSql {
    pub(super) fn from_request(request: &RequestRecordListRequest) -> Self {
        let mut params = SqlParams::default();
        let mut filters = Vec::new();
        add_request_filters(&mut filters, &mut params, request);
        Self {
            where_clause: where_clause(filters),
            values: params.values,
        }
    }

    pub(super) fn from_user_request(user_id: &str, request: &RequestRecordListRequest) -> Self {
        let mut params = SqlParams::default();
        let mut filters = Vec::new();
        add_eq_filter(&mut filters, &mut params, "r.user_id_snapshot", Some(user_id));
        add_usage_filters(&mut filters, &mut params, request);
        Self {
            where_clause: where_clause(filters),
            values: params.values,
        }
    }

    pub(super) fn push<T>(&mut self, value: T) -> String
    where
        T: Into<Value>,
    {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }
}

pub(super) fn pagination_value(field: &str, value: u64) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::Database(format!("request record {field} exceeds PostgreSQL integer range")))
}

fn add_request_filters(filters: &mut Vec<String>, params: &mut SqlParams, request: &RequestRecordListRequest) {
    add_status_filter(filters, params, request.status.as_deref());
    add_model_filter(filters, params, request.model_id.as_deref());
    add_eq_filter(filters, params, "r.provider_id", request.provider_id.as_deref());
    add_api_format_filter(filters, params, request.api_format.as_deref());
    add_type_filter(filters, request.type_filter.as_deref());
    add_search_filter(filters, params, request.search.as_deref());
}

fn add_usage_filters(filters: &mut Vec<String>, params: &mut SqlParams, request: &RequestRecordListRequest) {
    add_status_filter(filters, params, request.status.as_deref());
    add_model_filter(filters, params, request.model_id.as_deref());
    add_eq_filter(filters, params, "r.client_api_format", request.api_format.as_deref());
    add_type_filter(filters, request.type_filter.as_deref());
    add_usage_search_filter(filters, params, request.search.as_deref());
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

fn add_model_filter(filters: &mut Vec<String>, params: &mut SqlParams, value: Option<&str>) {
    if let Some(value) = non_empty(value) {
        let placeholder = params.push(value.to_owned());
        filters.push(format!("(r.global_model_id = {placeholder} OR r.model_name_snapshot = {placeholder})"));
    }
}

fn add_status_filter(filters: &mut Vec<String>, params: &mut SqlParams, value: Option<&str>) {
    match non_empty(value) {
        Some(STATUS_FILTER_ACTIVE) => add_active_status_filter(filters, params),
        Some(STATUS_FILTER_FAILOVER) => filters.push("r.has_failover = TRUE".into()),
        Some(STATUS_FILTER_RETRY) => filters.push("r.has_retry = TRUE".into()),
        Some(status) => add_eq_filter(filters, params, "r.status", Some(status)),
        None => {}
    }
}

fn add_active_status_filter(filters: &mut Vec<String>, params: &mut SqlParams) {
    let pending = params.push(STATUS_PENDING.to_owned());
    let streaming = params.push(STATUS_STREAMING.to_owned());
    filters.push(format!("r.status IN ({pending}, {streaming})"));
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

fn add_usage_search_filter(filters: &mut Vec<String>, params: &mut SqlParams, value: Option<&str>) {
    if let Some(value) = non_empty(value) {
        let placeholder = params.push(format!("%{}%", value.to_ascii_lowercase()));
        filters.push(format!("({})", usage_search_conditions(&placeholder).join(" OR ")));
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

fn usage_search_conditions(placeholder: &str) -> Vec<String> {
    [
        "LOWER(COALESCE(r.model_name_snapshot, ''))",
        "LOWER(COALESCE(r.global_model_id, ''))",
        "LOWER(r.client_api_format)",
        "LOWER(r.request_type)",
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
