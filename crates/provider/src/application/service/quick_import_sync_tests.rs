use sea_orm::{DatabaseBackend, MockDatabase};
use storage::Database;

use super::{
    quick_import_sync::SyncArgs,
    quick_import_sync::run_quick_import_sync,
    quick_import_sync_test_support::{
        DummyModels, TestCipher, TestImporter, count_source_events, count_source_run_updates, expires_at, no_model_bindings, provider_record, source_config,
        source_failure_event_record, sql_statements, sync_source_record,
    },
};
use crate::{
    application::{ProviderError, ProviderQuickImportSyncRunOptions},
    infra::StorageProviderRepository,
};

#[tokio::test]
async fn sync_counts_refresh_failure_and_continues_with_next_source() {
    let refreshed = source_config("https://sub2api-2.example", "new-access", "new-refresh", expires_at(180));
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![
            sync_source_record("provider-1", "bad-access", "bad-refresh", expires_at(10)),
            sync_source_record("provider-2", "old-access", "old-refresh", expires_at(20)),
        ]])
        .append_query_results([vec![provider_record("provider-1", "Provider 1"), provider_record("provider-2", "Provider 2")]])
        .append_query_results([[source_failure_event_record("event-1", "provider-1", "source-provider-1")]])
        .append_query_results([Vec::<storage::provider::record::provider_quick_import_keys::Model>::new()])
        .append_query_results([Vec::<storage::provider::record::provider_quick_import_keys::Model>::new()])
        .append_query_results([[sync_source_record("provider-1", "bad-access", "bad-refresh", expires_at(10))]])
        .append_query_results([[sync_source_record("provider-1", "bad-access", "bad-refresh", expires_at(10))]])
        .append_query_results([[sync_source_record("provider-2", "old-access", "old-refresh", expires_at(20))]])
        .append_query_results([[sync_source_record("provider-2", "new-access", "new-refresh", expires_at(180))]])
        .append_query_results([[provider_record("provider-2", "Provider 2")]])
        .append_query_results([Vec::<storage::provider::record::provider_quick_import_keys::Model>::new()])
        .append_query_results([Vec::<storage::provider::record::provider_quick_import_keys::Model>::new()])
        .append_query_results([no_model_bindings()])
        .append_query_results([[sync_source_record("provider-2", "new-access", "new-refresh", expires_at(180))]])
        .append_query_results([[sync_source_record("provider-2", "new-access", "new-refresh", expires_at(180))]])
        .into_connection();
    let repository = StorageProviderRepository::new(Database::new(connection.clone()));
    let importer = TestImporter::with_refresh_results([
        (
            "bad-access",
            Err(ProviderError::Infrastructure(
                "sub2api returned 401 Unauthorized: {\"code\":401,\"message\":\"invalid refresh token\",\"reason\":\"REFRESH_TOKEN_INVALID\"}".into(),
            )),
        ),
        ("old-access", Ok(refreshed)),
    ]);

    let result = run_quick_import_sync(
        SyncArgs {
            repository: &repository,
            models: &DummyModels,
            cipher: &TestCipher,
            importer: &importer,
        },
        ProviderQuickImportSyncRunOptions { limit: 20 },
    )
    .await;

    let statements = sql_statements(&connection);
    let report = result.unwrap();

    assert_eq!(report.scanned_count, 2);
    assert_eq!(report.synced_count, 1);
    assert_eq!(report.failed_count, 1);
    assert_eq!(report.disabled_key_count, 0);
    assert_eq!(report.updated_cost_count, 0);
    assert_eq!(count_source_events(&statements), 1);
    assert_eq!(count_source_run_updates(&statements), 2);
}
