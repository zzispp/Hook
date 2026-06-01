use super::{due_check::DUE_CHECKS_SQL, list_query::availability_statement};

#[test]
fn due_checks_sql_uses_skip_locked_claiming() {
    assert!(DUE_CHECKS_SQL.contains("FOR UPDATE SKIP LOCKED"));
    assert!(DUE_CHECKS_SQL.contains("UPDATE model_status_checks"));
    assert!(DUE_CHECKS_SQL.contains("locked_until"));
}

#[test]
fn availability_sql_reads_hourly_stats_not_raw_runs() {
    let statement = availability_statement(time::OffsetDateTime::UNIX_EPOCH, time::OffsetDateTime::UNIX_EPOCH, &["check-1".to_owned()]);
    assert!(statement.sql.contains("model_status_check_hourly_stats"));
    assert!(!statement.sql.contains("model_status_check_runs"));
}
