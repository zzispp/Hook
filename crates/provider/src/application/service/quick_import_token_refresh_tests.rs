use super::{
    quick_import_sync::SyncArgs,
    quick_import_token_refresh::run_quick_import_token_refresh,
    quick_import_token_refresh_test_support::{
        DummyModels, TestCipher, TestImporter, assert_no_source_update, assert_refresh_update_only_touches_credentials, assert_source_update_count, expires_at,
        provider_record, source_config, sql_statements, sync_source_record,
    },
};
use crate::{
    application::{ProviderError, ProviderQuickImportTokenRefreshRunOptions},
    infra::StorageProviderRepository,
};
use sea_orm::{DatabaseBackend, MockDatabase};
use storage::Database;

#[tokio::test]
async fn token_refresh_skips_still_valid_source_without_persisting() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![sync_source_record("provider-1", "old-access", "old-refresh", expires_at(120))]])
        .append_query_results([vec![provider_record("provider-1", "Provider 1")]])
        .into_connection();
    let repository = StorageProviderRepository::new(Database::new(connection.clone()));
    let report = run_quick_import_token_refresh(
        SyncArgs {
            repository: &repository,
            models: &DummyModels,
            cipher: &TestCipher,
            importer: &TestImporter::same_config(),
        },
        ProviderQuickImportTokenRefreshRunOptions {
            limit: 20,
            refresh_threshold_minutes: 60,
        },
    )
    .await
    .unwrap();

    assert_eq!(report.scanned_count, 1);
    assert_eq!(report.refreshed_count, 0);
    assert_eq!(report.skipped_count, 1);
    assert_eq!(report.failed_count, 0);
    let statements = sql_statements(&connection);
    assert_no_source_update(&statements);
}

#[tokio::test]
async fn token_refresh_persists_new_credentials_when_source_is_due() {
    let refreshed = source_config("https://sub2api-1.example", "new-access", "new-refresh", expires_at(240));
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![sync_source_record("provider-1", "old-access", "old-refresh", expires_at(20))]])
        .append_query_results([vec![provider_record("provider-1", "Provider 1")]])
        .append_query_results([[sync_source_record("provider-1", "old-access", "old-refresh", expires_at(20))]])
        .append_query_results([[sync_source_record("provider-1", "new-access", "new-refresh", expires_at(240))]])
        .append_query_results([[provider_record("provider-1", "Provider 1")]])
        .into_connection();
    let repository = StorageProviderRepository::new(Database::new(connection.clone()));
    let report = run_quick_import_token_refresh(
        SyncArgs {
            repository: &repository,
            models: &DummyModels,
            cipher: &TestCipher,
            importer: &TestImporter::with_result("old-access", Ok(refreshed)),
        },
        ProviderQuickImportTokenRefreshRunOptions {
            limit: 20,
            refresh_threshold_minutes: 60,
        },
    )
    .await
    .unwrap();

    assert_eq!(report.scanned_count, 1);
    assert_eq!(report.refreshed_count, 1);
    assert_eq!(report.skipped_count, 0);
    assert_eq!(report.failed_count, 0);
    let statements = sql_statements(&connection);
    assert_source_update_count(&statements, 1);
    assert_refresh_update_only_touches_credentials(&statements);
}

#[tokio::test]
async fn token_refresh_counts_individual_failures_and_continues() {
    let refreshed = source_config("https://sub2api-2.example", "new-access", "new-refresh", expires_at(180));
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![
            sync_source_record("provider-1", "bad-access", "bad-refresh", expires_at(10)),
            sync_source_record("provider-2", "old-access", "old-refresh", expires_at(20)),
        ]])
        .append_query_results([vec![provider_record("provider-1", "Provider 1"), provider_record("provider-2", "Provider 2")]])
        .append_query_results([[sync_source_record("provider-2", "old-access", "old-refresh", expires_at(20))]])
        .append_query_results([[sync_source_record("provider-2", "new-access", "new-refresh", expires_at(180))]])
        .append_query_results([[provider_record("provider-2", "Provider 2")]])
        .into_connection();
    let repository = StorageProviderRepository::new(Database::new(connection.clone()));
    let importer = TestImporter::with_results([
        ("bad-access", Err(ProviderError::Infrastructure("refresh failed".into()))),
        ("old-access", Ok(refreshed)),
    ]);
    let report = run_quick_import_token_refresh(
        SyncArgs {
            repository: &repository,
            models: &DummyModels,
            cipher: &TestCipher,
            importer: &importer,
        },
        ProviderQuickImportTokenRefreshRunOptions {
            limit: 20,
            refresh_threshold_minutes: 60,
        },
    )
    .await
    .unwrap();

    assert_eq!(report.scanned_count, 2);
    assert_eq!(report.refreshed_count, 1);
    assert_eq!(report.skipped_count, 0);
    assert_eq!(report.failed_count, 1);
    let statements = sql_statements(&connection);
    assert_source_update_count(&statements, 1);
}
