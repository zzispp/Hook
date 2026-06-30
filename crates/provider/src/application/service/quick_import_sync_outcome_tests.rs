use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, TieredPricingConfig},
    provider::{
        NewApiQuickImportConfig, ProviderQuickImportCostSyncMode, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus,
    },
};

use super::{quick_import_sync_bindings::BindingInfo, quick_import_sync_outcome::key_outcome};
use crate::application::{
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncSource, ProviderResult, UpstreamGroupRatio,
    UpstreamImportData, UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn overwrite_mode_updates_multiplier_and_costs() {
    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", true),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::Ok]);
    assert_eq!(outcome.costs.as_ref().unwrap()[0].price_per_request, Some(Decimal::new(2, 1)));
    let patch = outcome.patch("key-1".into());
    assert_eq!(patch.upstream_group_ratio, Some(Decimal::new(2, 0)));
    assert_eq!(patch.effective_cost_multiplier, Some(Decimal::new(2, 1)));
}

#[tokio::test]
async fn report_only_mode_marks_pending_without_updating_costs_or_multiplier() {
    let mut config = sync_config();
    config.cost_sync_mode = ProviderQuickImportCostSyncMode::ReportOnly;
    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(config),
        &source_config(),
        &snapshot("plus", true),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::CostPendingUpdate]);
    assert!(outcome.costs.is_none());
    let patch = outcome.patch("key-1".into());
    assert_eq!(patch.upstream_group_ratio, None);
    assert_eq!(patch.effective_cost_multiplier, None);
}

#[tokio::test]
async fn upstream_token_and_group_anomalies_are_reported() {
    let deleted = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &empty_snapshot(),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;
    let changed = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot("other", true),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;
    let removed = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot_without_group(),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(deleted.statuses, vec![ProviderQuickImportSyncStatus::UpstreamTokenDeleted]);
    assert_eq!(changed.statuses, vec![ProviderQuickImportSyncStatus::UpstreamGroupChanged]);
    assert_eq!(removed.statuses, vec![ProviderQuickImportSyncStatus::UpstreamGroupRemoved]);
}

#[tokio::test]
async fn sub2api_same_group_id_rename_keeps_ratio_lookup_working() {
    let mut key = key(Decimal::ONE);
    key.upstream_group_id = Some("2".into());
    key.upstream_group = Some("快速稳定分组2".into());

    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &sub2api_source(sync_config()),
        &sub2api_source_config(),
        &sub2api_snapshot(),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key,
    )
    .await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::Ok]);
    assert_eq!(outcome.observed_group_ratio, Some(Decimal::new(15, 2)));
    assert_eq!(outcome.patch("key-1".into()).upstream_group, Some(Some("快速稳定分组2（倍率上限0.15）".into())));
}

#[tokio::test]
async fn model_fetch_failure_and_removed_model_are_distinct_statuses() {
    let unavailable = key_outcome(
        &failing_importer(),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", true),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;
    let removed = key_outcome(
        &importer(&[]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", true),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(unavailable.statuses, vec![ProviderQuickImportSyncStatus::UpstreamKeyUnavailable]);
    assert_eq!(removed.statuses, vec![ProviderQuickImportSyncStatus::UpstreamModelRemoved]);
    assert_eq!(removed.missing_upstream_model_ids, vec!["upstream-gpt".to_owned()]);
    assert!(removed.upstream_models_snapshot.iter().all(|model| model.upstream_model_id != "upstream-gpt"));
}

#[tokio::test]
async fn missing_default_cost_is_reported() {
    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", true),
        &globals(None),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::CostUnavailable]);
    assert!(outcome.costs.is_none());
}

fn importer(models: &[&str]) -> TestImporter {
    TestImporter {
        models: models
            .iter()
            .map(|id| UpstreamImportModel {
                id: (*id).into(),
                supported_endpoint_types: vec![],
            })
            .collect(),
        fail: false,
    }
}

fn failing_importer() -> TestImporter {
    TestImporter { models: vec![], fail: true }
}

struct TestImporter {
    models: Vec<UpstreamImportModel>,
    fail: bool,
}

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("sync outcome tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("sync outcome tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        if self.fail {
            return Err(ProviderError::Infrastructure("upstream model list fetch failed".into()));
        }
        Ok(self.models.clone())
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

fn sub2api_source(sync_config: ProviderQuickImportSyncConfig) -> ProviderQuickImportSyncSource {
    ProviderQuickImportSyncSource {
        source_kind: types::provider::ProviderQuickImportSourceKind::Sub2api,
        base_url: "https://sub2api.example".into(),
        encrypted_system_access_token: String::new(),
        user_id: String::new(),
        ..source(sync_config)
    }
}

fn sync_config() -> ProviderQuickImportSyncConfig {
    ProviderQuickImportSyncConfig::default()
}

fn key(effective_cost_multiplier: Decimal) -> ProviderQuickImportSyncKey {
    ProviderQuickImportSyncKey {
        provider_id: "provider-1".into(),
        source_id: "source-1".into(),
        key_id: "key-1".into(),
        local_key_name: "生产主 Key".into(),
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_group_id: None,
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::ONE,
        effective_cost_multiplier,
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

fn sub2api_source_config() -> ProviderQuickImportSourceConfig {
    ProviderQuickImportSourceConfig::Sub2api(types::provider::Sub2ApiQuickImportConfig::Token(
        types::provider::Sub2ApiTokenQuickImportConfig {
            base_url: "https://sub2api.example".into(),
            auth_token: "auth-token".into(),
            refresh_token: "refresh-token".into(),
            token_expires_at: "2026-06-30T00:00:00Z".into(),
        },
    ))
}

fn snapshot(group: &str, is_active: bool) -> UpstreamSyncSnapshot {
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
            status: if is_active { "active".into() } else { "disabled".into() },
            is_active,
            group_id: None,
            group: Some(group.into()),
        }],
    }
}

fn sub2api_snapshot() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Sub2api,
        groups: BTreeMap::from([("2".into(), UpstreamGroupRatio::Fixed(Decimal::new(15, 2)))]),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status: "active".into(),
            is_active: true,
            group_id: Some("2".into()),
            group: Some("快速稳定分组2（倍率上限0.15）".into()),
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

fn snapshot_without_group() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::new(),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status: "active".into(),
            is_active: true,
            group_id: None,
            group: Some("plus".into()),
        }],
    }
}

fn globals(default_price_per_request: Option<Decimal>) -> BTreeMap<String, GlobalModelResponse> {
    BTreeMap::from([("global-gpt".into(), global_model(default_price_per_request))])
}
fn global_model(default_price_per_request: Option<Decimal>) -> GlobalModelResponse {
    GlobalModelResponse {
        id: "global-gpt".into(),
        name: "gpt".into(),
        display_name: "GPT".into(),
        is_active: true,
        default_price_per_request,
        default_tiered_pricing: TieredPricingConfig { tiers: vec![] },
        supported_capabilities: None,
        config: None,
        routing_profile_id: None,
        provider_count: None,
        active_provider_count: None,
        usage_count: None,
        created_at: String::new(),
        updated_at: None,
    }
}

fn bindings() -> BTreeMap<String, BindingInfo> {
    BTreeMap::from([("global-gpt".into(), BindingInfo { id: "provider-model-1".into() })])
}
