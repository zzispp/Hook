use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    provider::{ProviderQuickImportSyncEventRecordInput, ProviderStore, record::provider_quick_import_sync_events},
};
use types::provider::{
    ProviderQuickImportSyncEventPayload, ProviderQuickImportSyncEventSnapshotStatus, ProviderQuickImportSyncStatus, ProviderQuickImportUpstreamModelSnapshot,
};

#[tokio::test]
async fn create_quick_import_sync_events_persists_payload_json_column() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[event_record("event-1", Some(payload()))]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store.create_quick_import_sync_events(vec![event_input(Some(payload()))]).await.unwrap();

    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("INSERT INTO \"provider_quick_import_sync_events\""), "{sql}");
    assert!(sql.contains("\"payload_json\""), "{sql}");
}

#[tokio::test]
async fn quick_import_sync_event_detail_reports_available_snapshot_payload() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[event_record("event-1", Some(payload()))]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection));

    let detail = store.quick_import_sync_event_detail("event-1").await.unwrap().unwrap();

    assert_eq!(detail.snapshot_status, ProviderQuickImportSyncEventSnapshotStatus::Available);
    let payload = detail.payload.expect("payload should exist");
    assert_eq!(payload.provider_name, "OpenAI");
    assert_eq!(payload.missing_upstream_model_ids, vec!["missing-model".to_owned()]);
    assert_eq!(payload.upstream_models_snapshot[0].upstream_model_id, "gpt-5");
}

#[tokio::test]
async fn quick_import_sync_event_detail_reports_missing_snapshot_for_legacy_event() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[event_record("legacy-event", None)]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection));

    let detail = store.quick_import_sync_event_detail("legacy-event").await.unwrap().unwrap();

    assert_eq!(detail.snapshot_status, ProviderQuickImportSyncEventSnapshotStatus::Missing);
    assert!(detail.payload.is_none());
}

fn event_input(payload: Option<ProviderQuickImportSyncEventPayload>) -> ProviderQuickImportSyncEventRecordInput {
    ProviderQuickImportSyncEventRecordInput {
        provider_id: "provider-1".into(),
        source_id: "source-1".into(),
        key_id: Some("key-1".into()),
        status: ProviderQuickImportSyncStatus::UpstreamModelRemoved,
        title: "OpenAI 提供商，生产主 Key 密钥已导入模型在上游消失。已按策略禁用本地密钥".into(),
        detail: "已关联的上游模型缺失：missing-model。已按策略禁用本地密钥".into(),
        payload,
    }
}

fn event_record(id: &str, payload: Option<ProviderQuickImportSyncEventPayload>) -> provider_quick_import_sync_events::Model {
    provider_quick_import_sync_events::Model {
        id: id.into(),
        provider_id: "provider-1".into(),
        source_id: "source-1".into(),
        key_id: Some("key-1".into()),
        status: ProviderQuickImportSyncStatus::UpstreamModelRemoved.as_str().into(),
        title: "OpenAI 提供商，生产主 Key 密钥已导入模型在上游消失。已按策略禁用本地密钥".into(),
        detail: "已关联的上游模型缺失：missing-model。已按策略禁用本地密钥".into(),
        payload_json: payload.map(serde_json::to_value).transpose().unwrap(),
        created_at: ts(20),
    }
}

fn payload() -> ProviderQuickImportSyncEventPayload {
    ProviderQuickImportSyncEventPayload {
        provider_name: "OpenAI".into(),
        local_key_name: Some("生产主 Key".into()),
        upstream_token_name: Some("codex".into()),
        upstream_token_id: Some("1209".into()),
        status: ProviderQuickImportSyncStatus::UpstreamModelRemoved,
        anomaly_summary: "已关联的上游模型缺失：missing-model".into(),
        action_summary: "已按策略禁用本地密钥".into(),
        missing_upstream_model_ids: vec!["missing-model".into()],
        upstream_models_snapshot: vec![ProviderQuickImportUpstreamModelSnapshot {
            upstream_model_id: "gpt-5".into(),
            supported_endpoint_types: vec!["chat".into()],
        }],
    }
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}
