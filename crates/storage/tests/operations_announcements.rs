use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    operations::{
        AnnouncementRecord, NotificationStateRecord, OperationsStore,
    },
};

#[tokio::test]
async fn unread_announcements_query_uses_pinned_then_created_sort_and_skips_read_items() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![
            announcement_record("ann-3", true, true, 30, 30),
            announcement_record("ann-2", true, true, 20, 20),
            announcement_record("ann-1", false, true, 10, 10),
        ]])
        .append_query_results([Vec::<NotificationStateRecord>::new()])
        .append_query_results([[notification_state_record("ann-2", Some(40), None)]])
        .append_query_results([Vec::<NotificationStateRecord>::new()])
        .into_connection();
    let store = OperationsStore::new(Database::new(connection.clone()));

    let announcements = store.unread_announcements("user-1").await.unwrap();

    let ids = announcements.iter().map(|item| item.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["ann-3", "ann-1"]);

    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("\"announcements\".\"pinned\" DESC"), "{sql}");
    assert!(sql.contains("\"announcements\".\"created_at\" DESC"), "{sql}");
}

#[tokio::test]
async fn mark_notification_read_writes_state_and_next_unread_query_excludes_announcement() {
    let inserted_state = notification_state_record("ann-1", Some(30), None);
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<NotificationStateRecord>::new()])
        .append_query_results([[inserted_state.clone()]])
        .append_query_results([vec![announcement_record("ann-1", false, true, 10, 20)]])
        .append_query_results([[inserted_state]])
        .into_connection();
    let store = OperationsStore::new(Database::new(connection.clone()));

    store.mark_notification_read("user-1", "announcement", "ann-1").await.unwrap();
    let announcements = store.unread_announcements("user-1").await.unwrap();

    assert!(announcements.is_empty());

    let logs = connection.into_transaction_log();
    assert!(logs
        .iter()
        .flat_map(|tx| tx.statements())
        .any(|statement| statement.sql.contains("INSERT INTO \"notification_states\"")));
}

fn announcement_record(id: &str, pinned: bool, enabled: bool, created_at: i64, updated_at: i64) -> AnnouncementRecord {
    AnnouncementRecord {
        id: id.into(),
        title: format!("Announcement {id}"),
        content_markdown: "content".into(),
        announcement_type: "system".into(),
        pinned,
        enabled,
        created_by: "admin".into(),
        updated_by: "admin".into(),
        created_at: ts(created_at),
        updated_at: ts(updated_at),
    }
}

fn notification_state_record(source_id: &str, read_at: Option<i64>, deleted_at: Option<i64>) -> NotificationStateRecord {
    NotificationStateRecord {
        id: format!("state-{source_id}"),
        user_id: "user-1".into(),
        source_type: "announcement".into(),
        source_id: source_id.into(),
        read_at: read_at.map(ts),
        deleted_at: deleted_at.map(ts),
        created_at: ts(1),
        updated_at: ts(1),
    }
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}
