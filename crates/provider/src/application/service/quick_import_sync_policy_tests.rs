use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, TieredPricingConfig},
    provider::{
        NewApiQuickImportConfig, ProviderQuickImportGroupChangedAction, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig,
        ProviderQuickImportSyncStatus, ProviderQuickImportUpstreamAnomalyAction,
    },
};

use super::{quick_import_sync_bindings::BindingInfo, quick_import_sync_events::key_events, quick_import_sync_outcome::key_outcome};
use crate::application::{
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncSource, ProviderResult, UpstreamGroupRatio,
    UpstreamImportData, UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn per_anomaly_policy_controls_disable_behavior() {
    let mut report_only = sync_config();
    report_only.anomaly_actions.token_deleted = ProviderQuickImportUpstreamAnomalyAction::ReportOnly;

    let disabled = run_outcome(sync_config(), empty_snapshot()).await;
    let reported = run_outcome(report_only, empty_snapshot()).await;

    assert!(disabled.disable_key);
    assert!(!reported.disable_key);
}

#[tokio::test]
async fn key_model_fetch_failure_defaults_to_report_only_warning() {
    let sync_source = source(sync_config());
    let outcome = key_outcome(
        &FailingModelImporter,
        &sync_source,
        &source_config(),
        &snapshot("plus"),
        &globals(),
        &bindings(),
        &key(),
    )
    .await;
    let events = key_events(&sync_source, &key(), &outcome);

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::UpstreamKeyUnavailable]);
    assert!(!outcome.disable_key);
    assert_eq!(
        outcome.patch("key-1".into()).last_error,
        Some("infrastructure error: upstream /v1/models returned 429".into())
    );
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].title,
        "OpenAI 提供商，生产主 Key 密钥上游模型列表拉取失败。仅记录异常，未禁用本地密钥"
    );
    assert!(events[0].detail.contains("/v1/models"));
    assert!(events[0].detail.contains("未禁用本地密钥"));
    let payload = events[0].payload.as_ref().expect("payload should exist");
    assert_eq!(payload.provider_name, "OpenAI");
    assert_eq!(payload.local_key_name.as_deref(), Some("生产主 Key"));
    assert_eq!(payload.upstream_token_name.as_deref(), Some("codex"));
    assert_eq!(payload.upstream_token_id.as_deref(), Some("1209"));
}

#[tokio::test]
async fn cost_event_title_contains_full_key_context() {
    let config = sync_config();
    let sync_source = source(config.clone());
    let outcome = run_outcome(config, snapshot("plus")).await;
    let events = key_events(&sync_source, &key(), &outcome);
    let event = events.iter().find(|event| event.title.contains("成本倍率下降")).expect("cost event must exist");

    assert_eq!(event.title, "OpenAI 提供商，生产主 Key 密钥成本倍率下降");
    assert!(event.detail.contains("最终成本倍率从 1x 已更新为 0.2x"));
}

#[tokio::test]
async fn group_changed_sync_accepts_new_group_and_updates_cost() {
    let mut config = sync_config();
    config.anomaly_actions.group_changed = ProviderQuickImportGroupChangedAction::Sync;
    let outcome = run_outcome(config, snapshot("other")).await;

    let patch = outcome.patch("key-1".into());
    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::Ok]);
    assert_eq!(patch.upstream_group, Some(Some("other".into())));
    assert_eq!(patch.effective_cost_multiplier, Some(Decimal::new(1, 1)));
}

#[tokio::test]
async fn group_changed_disable_key_does_not_emit_group_synced_event() {
    let config = sync_config();
    let sync_source = source(config.clone());
    let outcome = run_outcome(config, snapshot("other")).await;
    let events = key_events(&sync_source, &key(), &outcome);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].status, ProviderQuickImportSyncStatus::UpstreamGroupChanged);
    assert!(events[0].title.contains("已按策略禁用本地密钥"));
    assert!(events[0].detail.contains("已按策略禁用本地密钥"));
}

#[tokio::test]
async fn group_changed_sync_emits_group_synced_event() {
    let mut config = sync_config();
    config.anomaly_actions.group_changed = ProviderQuickImportGroupChangedAction::Sync;
    let sync_source = source(config.clone());
    let outcome = run_outcome(config, snapshot("other")).await;
    let events = key_events(&sync_source, &key(), &outcome);

    assert!(events.iter().any(|event| event.title.contains("上游分组已同步")));
    assert!(!events.iter().any(|event| event.title.contains("同步异常")));
}

async fn run_outcome(config: ProviderQuickImportSyncConfig, snapshot: UpstreamSyncSnapshot) -> super::quick_import_sync_outcome::KeyOutcome {
    key_outcome(
        &TestImporter,
        &source(config),
        &source_config(),
        &snapshot,
        &globals(),
        &BTreeMap::from([("global-gpt".into(), BindingInfo { id: "provider-model-1".into() })]),
        &key(),
    )
    .await
}

fn bindings() -> BTreeMap<String, BindingInfo> {
    BTreeMap::from([("global-gpt".into(), BindingInfo { id: "provider-model-1".into() })])
}

struct TestImporter;

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("sync policy tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("sync policy tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        Ok(vec![UpstreamImportModel {
            id: "upstream-gpt".into(),
            supported_endpoint_types: vec![],
        }])
    }
}

struct FailingModelImporter;

#[async_trait]
impl UpstreamProviderImportSource for FailingModelImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("sync policy tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("sync policy tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        Err(ProviderError::Infrastructure("upstream /v1/models returned 429".into()))
    }
}

fn source(sync_config: ProviderQuickImportSyncConfig) -> ProviderQuickImportSyncSource {
    ProviderQuickImportSyncSource {
        id: "source-1".into(),
        provider_id: "provider-1".into(),
        provider_name: "OpenAI".into(),
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        base_url: "https://newapi.example".into(),
        encrypted_system_access_token: "enc".into(),
        email: String::new(),
        encrypted_password: String::new(),
        encrypted_auth_token: String::new(),
        encrypted_refresh_token: String::new(),
        token_expires_at: None,
        user_id: "737".into(),
        recharge_multiplier: Decimal::new(10, 0),
        sync_config,
        last_status: None,
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
    }
}

fn key() -> ProviderQuickImportSyncKey {
    ProviderQuickImportSyncKey {
        provider_id: "provider-1".into(),
        source_id: "source-1".into(),
        key_id: "key-1".into(),
        local_key_name: "生产主 Key".into(),
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::ONE,
        effective_cost_multiplier: Decimal::ONE,
        statuses: vec![ProviderQuickImportSyncStatus::Ok],
        model_mappings: vec![ProviderQuickImportSyncKeyModel {
            provider_model_id: "provider-model-1".into(),
            global_model_id: "global-gpt".into(),
            upstream_model_name: "upstream-gpt".into(),
            reasoning_effort: None,
        }],
    }
}

fn source_config() -> ProviderQuickImportSourceConfig {
    ProviderQuickImportSourceConfig::Newapi(NewApiQuickImportConfig {
        base_url: "https://newapi.example".into(),
        system_access_token: "system-token".into(),
        user_id: "737".into(),
    })
}

fn snapshot(group: &str) -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([
            ("plus".into(), UpstreamGroupRatio::Fixed(Decimal::new(2, 0))),
            ("other".into(), UpstreamGroupRatio::Fixed(Decimal::ONE)),
        ]),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status: "active".into(),
            is_active: true,
            group: Some(group.into()),
        }],
    }
}

fn empty_snapshot() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([("plus".into(), UpstreamGroupRatio::Fixed(Decimal::new(2, 0)))]),
        tokens: vec![],
    }
}

fn globals() -> BTreeMap<String, GlobalModelResponse> {
    BTreeMap::from([(
        "global-gpt".into(),
        GlobalModelResponse {
            id: "global-gpt".into(),
            name: "gpt".into(),
            display_name: "GPT".into(),
            is_active: true,
            default_price_per_request: Some(Decimal::ONE),
            default_tiered_pricing: TieredPricingConfig { tiers: vec![] },
            supported_capabilities: None,
            config: None,
            routing_profile_id: None,
            provider_count: None,
            active_provider_count: None,
            usage_count: None,
            created_at: String::new(),
            updated_at: None,
        },
    )])
}

fn sync_config() -> ProviderQuickImportSyncConfig {
    ProviderQuickImportSyncConfig::default()
}
