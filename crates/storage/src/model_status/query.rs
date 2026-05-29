use sea_orm::FromQueryResult;
use time::format_description::well_known::Rfc3339;
use types::model_status::{
    ModelStatusAvailability, ModelStatusCheckListResponse, ModelStatusCheckResponse, ModelStatusListRequest, ModelStatusTimelinePoint, ModelStatusValue,
};

use crate::{StorageError, StorageResult, model_status::entities::runs};

#[derive(Clone, Debug, FromQueryResult)]
pub(super) struct CheckRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) global_model_id: String,
    pub(super) model_name: String,
    pub(super) api_format: String,
    pub(super) api_token_id: String,
    pub(super) api_token_name: String,
    pub(super) interval_seconds: i64,
    pub(super) enabled: bool,
    pub(super) next_due_at: time::OffsetDateTime,
    pub(super) last_status: Option<String>,
    pub(super) last_checked_at: Option<time::OffsetDateTime>,
    pub(super) last_latency_ms: Option<i64>,
    pub(super) last_message: Option<String>,
    pub(super) created_at: time::OffsetDateTime,
    pub(super) updated_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, FromQueryResult)]
pub(super) struct AvailabilityRow {
    pub(super) check_id: String,
    pub(super) total_checks: i64,
    pub(super) available_checks: i64,
}

pub(super) fn list_response(checks: Vec<ModelStatusCheckResponse>) -> ModelStatusCheckListResponse {
    ModelStatusCheckListResponse { checks }
}

pub(super) fn response(row: CheckRow, availability: ModelStatusAvailability, timeline: Vec<ModelStatusTimelinePoint>) -> ModelStatusCheckResponse {
    ModelStatusCheckResponse {
        id: row.id,
        name: row.name,
        global_model_id: row.global_model_id,
        model_name: row.model_name,
        api_format: row.api_format,
        api_token_id: row.api_token_id,
        api_token_name: row.api_token_name,
        interval_seconds: row.interval_seconds,
        enabled: row.enabled,
        next_due_at: format_timestamp(row.next_due_at),
        last_status: row.last_status.and_then(|value| status_value(&value).ok()),
        last_checked_at: row.last_checked_at.map(format_timestamp),
        last_latency_ms: row.last_latency_ms,
        last_message: row.last_message,
        availability,
        timeline,
        created_at: format_timestamp(row.created_at),
        updated_at: format_timestamp(row.updated_at),
    }
}

pub(super) fn availability_for(rows: &[AvailabilityRow], check_id: &str) -> ModelStatusAvailability {
    let row = rows.iter().find(|row| row.check_id == check_id);
    let total = row.map(|row| row.total_checks).unwrap_or_default();
    let available = row.map(|row| row.available_checks).unwrap_or_default();
    let availability_pct = if total == 0 {
        None
    } else {
        Some(format!("{:.2}", available as f64 * 100.0 / total as f64))
    };
    ModelStatusAvailability {
        total_checks: total,
        available_checks: available,
        availability_pct,
    }
}

pub(super) fn empty_availability() -> ModelStatusAvailability {
    ModelStatusAvailability {
        total_checks: 0,
        available_checks: 0,
        availability_pct: None,
    }
}

pub(super) fn timeline_point(row: runs::Model) -> StorageResult<ModelStatusTimelinePoint> {
    Ok(ModelStatusTimelinePoint {
        status: status_value(&row.status)?,
        latency_ms: row.latency_ms,
        status_code: row.status_code,
        message: row.message,
        checked_at: format_timestamp(row.checked_at),
    })
}

pub(super) fn range_bounds(request: &ModelStatusListRequest) -> StorageResult<(time::OffsetDateTime, time::OffsetDateTime)> {
    let now = time::OffsetDateTime::now_utc();
    let date = now.date();
    match request.preset {
        types::model_status::ModelStatusRangePreset::Today => day_bounds(date),
        types::model_status::ModelStatusRangePreset::Yesterday => day_bounds(date - time::Duration::days(1)),
        types::model_status::ModelStatusRangePreset::Last7Days => Ok((now - time::Duration::days(7), now)),
        types::model_status::ModelStatusRangePreset::Last30Days => Ok((now - time::Duration::days(30), now)),
        types::model_status::ModelStatusRangePreset::Last90Days => Ok((now - time::Duration::days(90), now)),
        types::model_status::ModelStatusRangePreset::Custom => custom_bounds(request),
    }
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

fn custom_bounds(request: &ModelStatusListRequest) -> StorageResult<(time::OffsetDateTime, time::OffsetDateTime)> {
    let start = parse_date(
        request
            .start_date
            .as_deref()
            .ok_or_else(|| StorageError::Database("start_date is required".into()))?,
    )?;
    let end = parse_date(
        request
            .end_date
            .as_deref()
            .ok_or_else(|| StorageError::Database("end_date is required".into()))?,
    )?;
    let started_at = start.with_time(time::Time::MIDNIGHT).assume_utc();
    let ended_at = (end + time::Duration::days(1)).with_time(time::Time::MIDNIGHT).assume_utc();
    Ok((started_at, ended_at))
}

fn day_bounds(date: time::Date) -> StorageResult<(time::OffsetDateTime, time::OffsetDateTime)> {
    Ok((
        date.with_time(time::Time::MIDNIGHT).assume_utc(),
        (date + time::Duration::days(1)).with_time(time::Time::MIDNIGHT).assume_utc(),
    ))
}

fn parse_date(value: &str) -> StorageResult<time::Date> {
    time::Date::parse(value, &time::macros::format_description!("[year]-[month]-[day]"))
        .map_err(|error| StorageError::Database(format!("invalid date {value}: {error}")))
}

fn format_timestamp(value: time::OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("model status timestamp must format as RFC3339")
}
