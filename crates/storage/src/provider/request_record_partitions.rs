use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore,
    request_record_partition_columns::{REQUEST_CANDIDATE_PARTITION_TABLE, REQUEST_PAYLOAD_TABLE, REQUEST_RECORD_PARTITION_TABLE},
};

const DATE_COMPACT_LEN: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestPartitionMaintenanceOptions {
    pub now: time::OffsetDateTime,
    pub record_retention_days: i64,
    pub payload_retention_days: i64,
    pub future_days: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RequestPartitionMaintenanceResult {
    pub created_partitions: u64,
    pub dropped_partitions: u64,
}

struct PartitionFamily {
    parent: &'static str,
    prefix: &'static str,
    retention_days: i64,
}

#[derive(FromQueryResult)]
struct PartitionNameRow {
    partition_name: String,
}

pub async fn maintain_request_partitions(
    store: &ProviderStore,
    options: RequestPartitionMaintenanceOptions,
) -> StorageResult<RequestPartitionMaintenanceResult> {
    let mut result = RequestPartitionMaintenanceResult::default();
    for family in partition_families(options.record_retention_days, options.payload_retention_days) {
        result.created_partitions += create_family_partitions(store, &family, options.now, options.future_days).await?;
        result.dropped_partitions += drop_expired_family_partitions(store, &family, options.now).await?;
    }
    Ok(result)
}

fn partition_families(record_retention_days: i64, payload_retention_days: i64) -> [PartitionFamily; 3] {
    [
        PartitionFamily {
            parent: REQUEST_RECORD_PARTITION_TABLE,
            prefix: "request_records_partitioned",
            retention_days: record_retention_days,
        },
        PartitionFamily {
            parent: REQUEST_CANDIDATE_PARTITION_TABLE,
            prefix: "request_candidates_partitioned",
            retention_days: record_retention_days,
        },
        PartitionFamily {
            parent: REQUEST_PAYLOAD_TABLE,
            prefix: "request_payloads",
            retention_days: payload_retention_days,
        },
    ]
}

async fn create_family_partitions(store: &ProviderStore, family: &PartitionFamily, now: time::OffsetDateTime, future_days: i64) -> StorageResult<u64> {
    let mut created = 0;
    for offset in -1..=future_days {
        execute_ddl(store, &create_partition_sql(family, now.date() + time::Duration::days(offset))?).await?;
        created += 1;
    }
    Ok(created)
}

async fn drop_expired_family_partitions(store: &ProviderStore, family: &PartitionFamily, now: time::OffsetDateTime) -> StorageResult<u64> {
    let cutoff = compact_date(now.date() - time::Duration::days(family.retention_days));
    let mut dropped = 0;
    for partition_name in list_partition_names(store, family.parent).await? {
        if expired_partition(&partition_name, family.prefix, &cutoff) {
            execute_ddl(store, &drop_partition_sql(&partition_name)?).await?;
            dropped += 1;
        }
    }
    Ok(dropped)
}

async fn list_partition_names(store: &ProviderStore, parent: &str) -> StorageResult<Vec<String>> {
    let statement = Statement::from_sql_and_values(DbBackend::Postgres, partition_names_sql(), [Value::from(parent.to_owned())]);
    PartitionNameRow::find_by_statement(statement)
        .all(store.connection())
        .await
        .map(|rows| rows.into_iter().map(|row| row.partition_name).collect())
        .map_err(StorageError::from)
}

fn partition_names_sql() -> &'static str {
    "SELECT child.relname AS partition_name \
     FROM pg_inherits \
     JOIN pg_class parent ON pg_inherits.inhparent = parent.oid \
     JOIN pg_class child ON pg_inherits.inhrelid = child.oid \
     WHERE parent.relname = $1"
}

fn create_partition_sql(family: &PartitionFamily, day: time::Date) -> StorageResult<String> {
    let name = partition_name(family.prefix, day);
    let next_day = day + time::Duration::days(1);
    Ok(format!(
        "CREATE TABLE IF NOT EXISTS {} PARTITION OF {} FOR VALUES FROM ('{}') TO ('{}')",
        quote_ident(&name)?,
        quote_ident(family.parent)?,
        iso_date(day),
        iso_date(next_day)
    ))
}

fn drop_partition_sql(partition_name: &str) -> StorageResult<String> {
    Ok(format!("DROP TABLE IF EXISTS {}", quote_ident(partition_name)?))
}

fn expired_partition(partition_name: &str, prefix: &str, cutoff: &str) -> bool {
    partition_name
        .strip_prefix(&format!("{prefix}_"))
        .filter(|suffix| suffix.len() == DATE_COMPACT_LEN)
        .filter(|suffix| suffix.chars().all(|item| item.is_ascii_digit()))
        .is_some_and(|suffix| suffix < cutoff)
}

fn partition_name(prefix: &str, day: time::Date) -> String {
    format!("{}_{}", prefix, compact_date(day))
}

fn compact_date(day: time::Date) -> String {
    format!("{:04}{:02}{:02}", day.year(), u8::from(day.month()), day.day())
}

fn iso_date(day: time::Date) -> String {
    format!("{:04}-{:02}-{:02}", day.year(), u8::from(day.month()), day.day())
}

fn quote_ident(identifier: &str) -> StorageResult<String> {
    if identifier.chars().all(|item| item.is_ascii_alphanumeric() || item == '_') {
        return Ok(format!("\"{identifier}\""));
    }
    Err(StorageError::Database(format!("invalid partition identifier: {identifier}")))
}

async fn execute_ddl(store: &ProviderStore, sql: &str) -> StorageResult<()> {
    store
        .connection()
        .execute_raw(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{expired_partition, partition_name};

    #[test]
    fn partition_names_use_daily_suffix() {
        let day = time::Date::from_calendar_date(2026, time::Month::June, 9).unwrap();

        assert_eq!(partition_name("request_payloads", day), "request_payloads_20260609");
    }

    #[test]
    fn expired_partition_uses_prefix_and_date() {
        assert!(expired_partition("request_payloads_20260605", "request_payloads", "20260606"));
        assert!(!expired_partition("request_payloads_20260606", "request_payloads", "20260606"));
        assert!(!expired_partition("other_20260605", "request_payloads", "20260606"));
    }
}
