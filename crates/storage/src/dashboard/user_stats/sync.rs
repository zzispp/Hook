use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};

use crate::{StorageResult, provider::record::request_records};

use super::{GRANULARITY_DAY, GRANULARITY_HOUR, STATUS_CANCELLED, STATUS_FAILED, STATUS_SUCCESS};
use crate::dashboard::{latency_stage::StageLatencyContribution, scope::SqlParams, token_context};

pub async fn sync_user_usage_buckets(
    connection: &sea_orm::DatabaseConnection,
    old_record: &request_records::Model,
    new_record: &request_records::Model,
) -> StorageResult<()> {
    let old_contribution = contribution(old_record);
    let new_contribution = contribution(new_record);
    if old_contribution.is_none() && new_contribution.is_none() {
        return Ok(());
    }
    let tx = connection.begin().await?;
    apply_record_delta(&tx, old_contribution.as_ref(), -1).await?;
    apply_record_delta(&tx, new_contribution.as_ref(), 1).await?;
    tx.commit().await?;
    Ok(())
}

async fn apply_record_delta<C>(connection: &C, contribution: Option<&BucketContribution>, multiplier: i64) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let Some(contribution) = contribution else {
        return Ok(());
    };
    upsert_bucket_delta(connection, contribution, GRANULARITY_HOUR, hour_bounds(contribution.created_at), multiplier).await?;
    upsert_bucket_delta(connection, contribution, GRANULARITY_DAY, day_bounds(contribution.created_at), multiplier).await
}

async fn upsert_bucket_delta<C>(
    connection: &C,
    contribution: &BucketContribution,
    granularity: &str,
    bounds: BucketBounds,
    multiplier: i64,
) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let mut params = SqlParams::new();
    let sql = format!(
        "INSERT INTO dashboard_user_usage_buckets \
        (id, bucket_granularity, bucket_started_at, bucket_ended_at, user_id, username, request_count, success_count, failed_count, total_tokens, total_cost, \
        total_latency_ms, first_output_total_ms, first_output_sample_count, created_at, updated_at) \
        VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) \
        ON CONFLICT (bucket_granularity, bucket_started_at, user_id) DO UPDATE SET \
        username = EXCLUDED.username, \
        request_count = dashboard_user_usage_buckets.request_count + EXCLUDED.request_count, \
        success_count = dashboard_user_usage_buckets.success_count + EXCLUDED.success_count, \
        failed_count = dashboard_user_usage_buckets.failed_count + EXCLUDED.failed_count, \
        total_tokens = dashboard_user_usage_buckets.total_tokens + EXCLUDED.total_tokens, \
        total_cost = dashboard_user_usage_buckets.total_cost + EXCLUDED.total_cost, \
        total_latency_ms = dashboard_user_usage_buckets.total_latency_ms + EXCLUDED.total_latency_ms, \
        first_output_total_ms = dashboard_user_usage_buckets.first_output_total_ms + EXCLUDED.first_output_total_ms, \
        first_output_sample_count = dashboard_user_usage_buckets.first_output_sample_count + EXCLUDED.first_output_sample_count, \
        updated_at = EXCLUDED.updated_at",
        params.push(uuid::Uuid::now_v7().to_string()),
        params.push(granularity.to_owned()),
        params.push(bounds.started_at),
        params.push(bounds.ended_at),
        params.push(contribution.user_id.clone()),
        params.push(contribution.username.clone()),
        params.push(multiplier),
        params.push(contribution.success_count * multiplier),
        params.push(contribution.failed_count * multiplier),
        params.push(contribution.total_tokens * multiplier),
        params.push(contribution.total_cost * Decimal::from(multiplier)),
        params.push(contribution.total_latency_ms * multiplier),
        params.push(contribution.first_output_total_ms * multiplier),
        params.push(contribution.first_output_sample_count * multiplier),
        params.push(time::OffsetDateTime::now_utc()),
        params.push(time::OffsetDateTime::now_utc())
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

fn contribution(record: &request_records::Model) -> Option<BucketContribution> {
    if !is_terminal_status(&record.status) {
        return None;
    }
    let user_id = record.user_id_snapshot.as_ref()?.trim();
    if user_id.is_empty() {
        return None;
    }
    let stage = StageLatencyContribution::new(record.response_headers_time_ms, record.first_sse_event_time_ms, record.first_output_time_ms);
    Some(BucketContribution {
        user_id: user_id.to_owned(),
        username: record.username_snapshot.clone(),
        success_count: i64::from(record.status == STATUS_SUCCESS),
        failed_count: i64::from(record.status == STATUS_FAILED || record.status == STATUS_CANCELLED),
        total_tokens: token_context::total_tokens(record),
        total_cost: record.total_cost.unwrap_or(Decimal::ZERO),
        total_latency_ms: record.total_latency_ms.unwrap_or_default(),
        first_output_total_ms: StageLatencyContribution::total(stage.first_output_ms),
        first_output_sample_count: StageLatencyContribution::sample_count(stage.first_output_ms),
        created_at: record.created_at,
    })
}

fn hour_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let started_at = value
        .replace_minute(0)
        .and_then(|v| v.replace_second(0))
        .and_then(|v| v.replace_nanosecond(0))
        .unwrap_or(value);
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::hours(1),
    }
}

fn day_bounds(value: time::OffsetDateTime) -> BucketBounds {
    let date = value.date();
    let started_at = date.midnight().assume_utc();
    BucketBounds {
        started_at,
        ended_at: started_at + time::Duration::days(1),
    }
}

fn is_terminal_status(status: &str) -> bool {
    status == STATUS_SUCCESS || status == STATUS_FAILED || status == STATUS_CANCELLED
}

#[derive(Clone, Debug)]
struct BucketContribution {
    user_id: String,
    username: Option<String>,
    success_count: i64,
    failed_count: i64,
    total_tokens: i64,
    total_cost: Decimal,
    total_latency_ms: i64,
    first_output_total_ms: i64,
    first_output_sample_count: i64,
    created_at: time::OffsetDateTime,
}

#[derive(Clone, Copy, Debug)]
struct BucketBounds {
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
}
