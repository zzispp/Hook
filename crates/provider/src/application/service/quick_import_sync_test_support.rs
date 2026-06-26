use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::collections::BTreeMap;
use storage::model::provider_models;
use storage::provider::record::{provider_quick_import_sources, provider_quick_import_sync_events, providers};
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderQuickImportSourceConfig, ProviderQuickImportSourceKind, ProviderQuickImportSyncStatus, Sub2ApiQuickImportConfig, Sub2ApiTokenQuickImportConfig,
    },
};

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderResult, SecretCipher, UpstreamGroupRatio, UpstreamImportData, UpstreamImportModel, UpstreamProviderImportSource,
    UpstreamSyncSnapshot,
};

pub(super) struct DummyModels;

#[async_trait]
impl GlobalModelCatalog for DummyModels {
    async fn global_model_exists(&self, _id: &str) -> ProviderResult<bool> {
        unreachable!("quick import sync tests do not query individual global model existence")
    }

    async fn list_global_models(&self) -> ProviderResult<Vec<GlobalModelResponse>> {
        Ok(Vec::new())
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
    refresh_results: BTreeMap<String, TestRefreshResult>,
}

impl TestImporter {
    pub(super) fn with_refresh_results<const N: usize>(entries: [(&str, ProviderResult<ProviderQuickImportSourceConfig>); N]) -> Self {
        Self {
            refresh_results: entries
                .into_iter()
                .map(|(token, result)| (token.into(), TestRefreshResult::from_result(result)))
                .collect(),
        }
    }
}

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("quick import sync tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        let ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(_config)) = source else {
            unreachable!("quick import sync tests only use sub2api token sources")
        };
        Ok(UpstreamSyncSnapshot {
            source_kind: ProviderQuickImportSourceKind::Sub2api,
            groups: BTreeMap::from([("default".into(), UpstreamGroupRatio::Fixed(rust_decimal::Decimal::ONE))]),
            tokens: Vec::new(),
        })
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        unreachable!("quick import sync tests do not fetch token models")
    }

    async fn refreshed_source_config(&self, source: &ProviderQuickImportSourceConfig) -> ProviderResult<Option<ProviderQuickImportSourceConfig>> {
        let ProviderQuickImportSourceConfig::Sub2api(Sub2ApiQuickImportConfig::Token(config)) = source else {
            unreachable!("quick import sync tests only use sub2api token sources")
        };
        let Some(result) = self.refresh_results.get(config.auth_token.as_str()) else {
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
        auto_sync_enabled: true,
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
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    }
}

pub(super) fn source_failure_event_record(id: &str, provider_id: &str, source_id: &str) -> provider_quick_import_sync_events::Model {
    provider_quick_import_sync_events::Model {
        id: id.into(),
        provider_id: provider_id.into(),
        source_id: source_id.into(),
        key_id: None,
        status: ProviderQuickImportSyncStatus::SourceFetchFailed.as_str().into(),
        title: "快捷导入同步失败".into(),
        detail: "同步来源拉取失败".into(),
        payload_json: None,
        created_at: fixed_now(),
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

pub(super) fn count_source_run_updates(statements: &[String]) -> usize {
    statements
        .iter()
        .filter(|sql| sql.contains("UPDATE \"provider_quick_import_sources\" SET \"last_status\" ="))
        .count()
}

pub(super) fn count_source_events(statements: &[String]) -> usize {
    statements
        .iter()
        .filter(|sql| sql.contains("INSERT INTO \"provider_quick_import_sync_events\""))
        .count()
}

pub(super) fn no_model_bindings() -> Vec<provider_models::Model> {
    Vec::new()
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
