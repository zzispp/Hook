use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::{
    StorageError, StorageResult,
    model_status::{ModelStatusRunRecordInput, ModelStatusRunStatusValue},
};

pub(super) async fn upsert_hourly_stat<C>(connection: &C, id: &str, record: &ModelStatusRunRecordInput, now: time::OffsetDateTime) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let bucket = hourly_bucket(record.checked_at)?;
    let available = matches!(record.status, ModelStatusRunStatusValue::Operational | ModelStatusRunStatusValue::Degraded) as i64;
    let degraded = (record.status == ModelStatusRunStatusValue::Degraded) as i64;
    let failed = (record.status == ModelStatusRunStatusValue::Failed) as i64;
    let error = (record.status == ModelStatusRunStatusValue::Error) as i64;
    let sql = "INSERT INTO model_status_check_hourly_stats (id, check_id, bucket_started_at, total_count, available_count, degraded_count, failed_count, error_count, latency_sum_ms, created_at, updated_at) \
        VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, $9) \
        ON CONFLICT (check_id, bucket_started_at) DO UPDATE SET \
        total_count = model_status_check_hourly_stats.total_count + EXCLUDED.total_count, \
        available_count = model_status_check_hourly_stats.available_count + EXCLUDED.available_count, \
        degraded_count = model_status_check_hourly_stats.degraded_count + EXCLUDED.degraded_count, \
        failed_count = model_status_check_hourly_stats.failed_count + EXCLUDED.failed_count, \
        error_count = model_status_check_hourly_stats.error_count + EXCLUDED.error_count, \
        latency_sum_ms = model_status_check_hourly_stats.latency_sum_ms + EXCLUDED.latency_sum_ms, \
        updated_at = EXCLUDED.updated_at";
    connection
        .execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                id.to_owned().into(),
                record.check_id.clone().into(),
                bucket.into(),
                available.into(),
                degraded.into(),
                failed.into(),
                error.into(),
                record.latency_ms.unwrap_or_default().into(),
                now.into(),
            ],
        ))
        .await?;
    Ok(())
}

fn hourly_bucket(value: time::OffsetDateTime) -> StorageResult<time::OffsetDateTime> {
    value
        .replace_minute(0)
        .and_then(|value| value.replace_second(0))
        .and_then(|value| value.replace_nanosecond(0))
        .map_err(|error| StorageError::Database(format!("invalid hourly bucket: {error}")))
}
