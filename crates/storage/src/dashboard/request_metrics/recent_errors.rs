use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::{StorageResult, dashboard::scope::SqlParams, provider::record::request_records};

use super::common::{clean_optional, request_error_category};

pub(super) async fn sync_recent_error_snapshot<C>(connection: &C, record: &request_records::Model) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    match request_error_category(record) {
        Some(category) => upsert_recent_error(connection, record, category).await,
        None => delete_recent_error(connection, &record.request_id).await,
    }
}

async fn upsert_recent_error<C>(connection: &C, record: &request_records::Model, category: String) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let now = time::OffsetDateTime::now_utc();
    let mut params = SqlParams::new();
    let sql = format!(
        "INSERT INTO dashboard_recent_error_snapshots \
        (request_id, created_at, provider_id, provider_name, model, status_code, error_type, error_message, error_category, \
        response_headers_ms, first_output_ms, latency_ms, ttfb_ms, updated_at) \
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) \
        ON CONFLICT (request_id) DO UPDATE SET \
        created_at = EXCLUDED.created_at, provider_id = EXCLUDED.provider_id, provider_name = EXCLUDED.provider_name, model = EXCLUDED.model, \
        status_code = EXCLUDED.status_code, error_type = EXCLUDED.error_type, error_message = EXCLUDED.error_message, \
        error_category = EXCLUDED.error_category, response_headers_ms = EXCLUDED.response_headers_ms, first_output_ms = EXCLUDED.first_output_ms, \
        latency_ms = EXCLUDED.latency_ms, ttfb_ms = EXCLUDED.ttfb_ms, updated_at = EXCLUDED.updated_at",
        params.push(record.request_id.clone()),
        params.push(record.created_at),
        params.push(record.provider_id.clone()),
        params.push(record.provider_name_snapshot.clone()),
        params.push(error_model(record)),
        params.push(record.client_status_code),
        params.push(record.client_error_type.clone()),
        params.push(record.client_error_message.clone()),
        params.push(category),
        params.push(record.response_headers_time_ms),
        params.push(record.first_output_time_ms),
        params.push(record.total_latency_ms),
        params.push(record.first_byte_time_ms),
        params.push(now)
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

async fn delete_recent_error<C>(connection: &C, request_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let mut params = SqlParams::new();
    let sql = format!(
        "DELETE FROM dashboard_recent_error_snapshots WHERE request_id = {}",
        params.push(request_id.to_owned())
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

fn error_model(record: &request_records::Model) -> Option<String> {
    clean_optional(record.model_name_snapshot.clone()).or_else(|| clean_optional(record.global_model_id.clone()))
}
