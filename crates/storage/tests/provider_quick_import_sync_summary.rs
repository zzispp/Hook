use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    provider::{
        ProviderStore,
        record::{provider_api_keys, provider_quick_import_keys, provider_quick_import_sources, providers},
    },
};
use types::provider::{
    ProviderListRequest, ProviderOrigin, ProviderQuickImportSyncIssueScope, ProviderQuickImportSyncIssueSeverity, ProviderQuickImportSyncStatus,
};

#[tokio::test]
async fn provider_list_adds_current_quick_import_key_summary() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([mixed_provider_records()])
            .append_query_results([source_records("quick-import-provider", None)])
            .append_query_results([source_records("quick-import-provider", None)])
            .append_query_results([key_records(
                "quick-import-provider",
                "source-quick-import-provider",
                "quick-import-key",
                [ProviderQuickImportSyncStatus::Ok, ProviderQuickImportSyncStatus::UpstreamKeyUnavailable],
            )])
            .append_query_results([api_key_records("quick-import-provider", "quick-import-key", "Imported key")])
            .into_connection(),
    );
    let response = ProviderStore::new(database).list_providers(list_request()).await.unwrap();
    let manual = provider_by_origin(&response.providers, ProviderOrigin::Manual);
    let quick_import = provider_by_origin(&response.providers, ProviderOrigin::QuickImport);
    let summary = quick_import.quick_import_sync_summary.as_ref().unwrap();

    assert!(manual.quick_import_sync_summary.is_none());
    assert_eq!(summary.severity, ProviderQuickImportSyncIssueSeverity::Warning);
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.affected_key_count, 1);
    assert_eq!(summary.issues[0].scope, ProviderQuickImportSyncIssueScope::Key);
    assert_eq!(summary.issues[0].status, ProviderQuickImportSyncStatus::UpstreamKeyUnavailable);
    assert_eq!(summary.issues[0].key_name.as_deref(), Some("Imported key"));
}

#[tokio::test]
async fn provider_list_deduplicates_source_fetch_failure_summary() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![provider_record("quick-import-provider", "Quick Import", ProviderOrigin::QuickImport, 1)]])
            .append_query_results([source_records("quick-import-provider", Some(ProviderQuickImportSyncStatus::SourceFetchFailed))])
            .append_query_results([source_records("quick-import-provider", Some(ProviderQuickImportSyncStatus::SourceFetchFailed))])
            .append_query_results([key_records(
                "quick-import-provider",
                "source-quick-import-provider",
                "quick-import-key",
                [ProviderQuickImportSyncStatus::SourceFetchFailed],
            )])
            .append_query_results([api_key_records("quick-import-provider", "quick-import-key", "Imported key")])
            .into_connection(),
    );
    let response = ProviderStore::new(database).list_providers(list_request()).await.unwrap();
    let summary = response.providers[0].quick_import_sync_summary.as_ref().unwrap();

    assert_eq!(summary.severity, ProviderQuickImportSyncIssueSeverity::Error);
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.affected_key_count, 0);
    assert_eq!(summary.issues[0].scope, ProviderQuickImportSyncIssueScope::Source);
    assert_eq!(summary.issues[0].status, ProviderQuickImportSyncStatus::SourceFetchFailed);
    assert_eq!(summary.issues[0].message.as_deref(), Some("source fetch failed"));
}

fn provider_by_origin(providers: &[types::provider::Provider], origin: ProviderOrigin) -> &types::provider::Provider {
    providers.iter().find(|provider| provider.provider_origin == origin).unwrap()
}

fn list_request() -> ProviderListRequest {
    ProviderListRequest {
        limit: 100,
        ..Default::default()
    }
}

fn mixed_provider_records() -> Vec<providers::Model> {
    vec![
        provider_record("manual-provider", "Manual", ProviderOrigin::Manual, 1),
        provider_record("quick-import-provider", "Quick Import", ProviderOrigin::QuickImport, 2),
    ]
}

fn provider_record(id: &str, name: &str, origin: ProviderOrigin, priority: i32) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: name.into(),
        provider_type: "custom".into(),
        provider_origin: origin.as_str().into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_response_headers_timeout_seconds: Some(60.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_first_token_timeout_seconds: Some(45.0),
        stream_idle_timeout_seconds: Some(30.0),
        priority,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }
}

fn source_records(provider_id: &str, last_status: Option<ProviderQuickImportSyncStatus>) -> Vec<provider_quick_import_sources::Model> {
    vec![provider_quick_import_sources::Model {
        id: format!("source-{provider_id}"),
        provider_id: provider_id.into(),
        source_kind: "newapi".into(),
        base_url: "https://example.test".into(),
        encrypted_system_access_token: "encrypted".into(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: String::new(),
        encrypted_refresh_token: String::new(),
        token_expires_at: None,
        user_id: "user-1".into(),
        recharge_multiplier: rust_decimal::Decimal::ONE,
        auto_sync_enabled: true,
        cost_sync_mode: "overwrite".into(),
        upstream_anomaly_action: "report_only".into(),
        token_deleted_action: "report_only".into(),
        token_disabled_action: "report_only".into(),
        group_removed_action: "report_only".into(),
        group_changed_action: "sync".into(),
        key_unavailable_action: "report_only".into(),
        model_removed_action: "report_only".into(),
        fetch_failure_action: "report_only".into(),
        fetch_failure_disable_threshold: 3,
        last_status: last_status.map(|status| status.as_str().to_owned()),
        last_error: Some("source fetch failed".into()),
        last_synced_at: Some(now()),
        consecutive_failures: 1,
        created_at: now(),
        updated_at: now(),
    }]
}

fn key_records<const N: usize>(
    provider_id: &str,
    source_id: &str,
    key_id: &str,
    statuses: [ProviderQuickImportSyncStatus; N],
) -> Vec<provider_quick_import_keys::Model> {
    vec![provider_quick_import_keys::Model {
        id: format!("quick-import-{key_id}"),
        provider_id: provider_id.into(),
        source_id: source_id.into(),
        key_id: key_id.into(),
        upstream_token_id: "upstream-token".into(),
        upstream_token_name: "Upstream token".into(),
        upstream_masked_key: "sk-***".into(),
        upstream_group_id: None,
        upstream_group: Some("default".into()),
        upstream_group_ratio: rust_decimal::Decimal::ONE,
        effective_cost_multiplier: rust_decimal::Decimal::ONE,
        sync_statuses: serde_json::to_string(&statuses.to_vec()).unwrap(),
        last_sync_error: Some("key sync failed".into()),
        last_synced_at: Some(now()),
        created_at: now(),
        updated_at: now(),
    }]
}

fn api_key_records(provider_id: &str, key_id: &str, name: &str) -> Vec<provider_api_keys::Model> {
    vec![provider_api_keys::Model {
        id: key_id.into(),
        provider_id: provider_id.into(),
        name: name.into(),
        api_formats: "[]".into(),
        allowed_model_ids: "[]".into(),
        encrypted_api_key: "encrypted".into(),
        note: None,
        internal_priority: 0,
        global_priority_by_format: "{}".into(),
        rpm_limit: None,
        learned_rpm_limit: None,
        cache_ttl_minutes: 0,
        max_probe_interval_minutes: 0,
        time_range_enabled: false,
        time_range_start: None,
        time_range_end: None,
        health_by_format: None,
        circuit_breaker_by_format: None,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }]
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}
