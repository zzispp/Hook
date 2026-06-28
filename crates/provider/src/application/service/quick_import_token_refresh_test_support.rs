use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::collections::BTreeMap;
use storage::provider::record::{provider_quick_import_sources, providers};
use types::provider::{ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind, Sub2ApiQuickImportConfig, Sub2ApiTokenQuickImportConfig};

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderResult, SecretCipher, UpstreamImportData, UpstreamImportModel, UpstreamProviderImportSource,
    UpstreamSyncSnapshot,
};

pub(super) struct DummyModels;

#[async_trait]
impl GlobalModelCatalog for DummyModels {
    async fn global_model_exists(&self, _id: &str) -> ProviderResult<bool> {
        unreachable!("token refresh does not query model catalog")
    }

    async fn list_global_models(&self) -> ProviderResult<Vec<types::model::GlobalModelResponse>> {
        unreachable!("token refresh does not query model catalog")
    }
}

pub(super) struct TestCipher;

impl SecretCipher for TestCipher {
    fn encrypt_provider_key(&self, plaintext: &str) -> ProviderResult<String> {
        Ok(format!("enc:{plaintext}"))
    }

    fn decrypt_provider_key(&self, ciphertext: &str) -> ProviderResult<String> {
        Ok(ciphertext.strip_prefix("enc:").unwrap_or(ciphertext).to_owned())
    }
}

pub(super) struct TestImporter {
    results: BTreeMap<String, TestRefreshResult>,
}

impl TestImporter {
    pub(super) fn same_config() -> Self {
        Self { results: BTreeMap::new() }
    }

    pub(super) fn with_result(token: &str, result: ProviderResult<ProviderQuickImportSourceConfig>) -> Self {
        Self {
            results: BTreeMap::from([(token.into(), TestRefreshResult::from_result(result))]),
        }
    }

    pub(super) fn with_results<const N: usize>(entries: [(&str, ProviderResult<ProviderQuickImportSourceConfig>); N]) -> Self {
        Self {
            results: entries
                .into_iter()
                .map(|(token, result)| (token.into(), TestRefreshResult::from_result(result)))
                .collect(),
        }
    }
}

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("token refresh tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("token refresh tests do not fetch sync snapshots")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        unreachable!("token refresh tests do not fetch token models")
    }

    async fn refreshed_source_config_with_threshold(
        &self,
        source: &ProviderQuickImportSourceConfig,
        _refresh_threshold_minutes: i64,
    ) -> ProviderResult<Option<ProviderQuickImportSourceConfig>> {
        let ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(config)) = source else {
            unreachable!("token refresh tests only use sub2api token sources")
        };
        let Some(result) = self.results.get(config.auth_token.as_str()) else {
            return Ok(Some(source.clone()));
        };
        result.clone_result().map(Some)
    }
}

enum TestRefreshResult {
    Ok(ProviderQuickImportSourceConfig),
    Err(String),
}

impl TestRefreshResult {
    fn from_result(result: ProviderResult<ProviderQuickImportSourceConfig>) -> Self {
        match result {
            Ok(config) => Self::Ok(config),
            Err(error) => Self::Err(error.to_string()),
        }
    }

    fn clone_result(&self) -> ProviderResult<ProviderQuickImportSourceConfig> {
        match self {
            Self::Ok(config) => Ok(config.clone()),
            Self::Err(message) => Err(ProviderError::Infrastructure(message.clone())),
        }
    }
}

pub(super) fn assert_no_source_update(statements: &[String]) {
    assert!(statements.iter().all(|sql| !sql.contains("UPDATE \"provider_quick_import_sources\"")));
}

pub(super) fn assert_source_update_count(statements: &[String], expected: usize) {
    assert_eq!(
        statements.iter().filter(|sql| sql.contains("UPDATE \"provider_quick_import_sources\"")).count(),
        expected
    );
}

pub(super) fn assert_refresh_update_only_touches_credentials(statements: &[String]) {
    let update_sql = statements
        .iter()
        .find(|sql| sql.contains("UPDATE \"provider_quick_import_sources\""))
        .expect("expected quick import source update");
    let assignments = update_sql
        .split(" SET ")
        .nth(1)
        .and_then(|sql| sql.split(" WHERE ").next())
        .expect("expected SET clause in source update");
    assert!(
        assignments.contains("\"encrypted_auth_token\"") && assignments.contains("\"encrypted_refresh_token\"") && assignments.contains("\"token_expires_at\"")
    );
    assert!(
        !assignments.contains("\"base_url\"")
            && !assignments.contains("\"email\"")
            && !assignments.contains("\"encrypted_password\"")
            && !assignments.contains("\"last_status\"")
            && !assignments.contains("\"last_error\"")
            && !assignments.contains("\"last_synced_at\"")
    );
}

pub(super) fn source_config(base_url: &str, auth_token: &str, refresh_token: &str, token_expires_at: time::OffsetDateTime) -> ProviderQuickImportSourceConfig {
    ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(Sub2ApiTokenQuickImportConfig {
        base_url: base_url.into(),
        auth_token: auth_token.into(),
        refresh_token: refresh_token.into(),
        token_expires_at: token_expires_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
    }))
}

pub(super) fn sync_source_record(
    provider_id: &str,
    auth_token: &str,
    refresh_token: &str,
    token_expires_at: time::OffsetDateTime,
) -> provider_quick_import_sources::Model {
    provider_quick_import_sources::Model {
        id: format!("source-{provider_id}"),
        provider_id: provider_id.into(),
        source_kind: ProviderQuickImportSourceKind::Sub2api.as_str().to_owned(),
        base_url: format!("https://sub2api-{}.example", provider_id.trim_start_matches("provider-")),
        encrypted_system_access_token: String::new(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: format!("enc:{auth_token}"),
        encrypted_refresh_token: format!("enc:{refresh_token}"),
        token_expires_at: Some(token_expires_at),
        user_id: String::new(),
        recharge_multiplier: rust_decimal::Decimal::ONE,
        auto_sync_enabled: false,
        cost_sync_mode: "overwrite".into(),
        upstream_anomaly_action: "disable_key".into(),
        token_deleted_action: "disable_key".into(),
        token_disabled_action: "disable_key".into(),
        group_removed_action: "disable_key".into(),
        group_changed_action: "disable_key".into(),
        key_unavailable_action: "disable_key".into(),
        model_removed_action: "disable_key".into(),
        fetch_failure_action: "report_only".into(),
        fetch_failure_disable_threshold: 3,
        last_status: None,
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    }
}

pub(super) fn provider_record(id: &str, name: &str) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: name.into(),
        provider_type: "custom".into(),
        provider_origin: "quick_import".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_first_output_timeout_seconds: Some(45.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    }
}

pub(super) fn sql_statements(connection: &DatabaseConnection) -> Vec<String> {
    connection
        .clone()
        .into_transaction_log()
        .into_iter()
        .flat_map(|transaction: sea_orm::Transaction| transaction.statements().iter().map(|statement| statement.sql.clone()).collect::<Vec<_>>())
        .collect()
}

pub(super) fn expires_at(minutes_from_now: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc() + time::Duration::minutes(minutes_from_now)
}

fn fixed_now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::June, 23)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}
