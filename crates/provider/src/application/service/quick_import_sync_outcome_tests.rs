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
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncSource, ProviderResult, UpstreamImportData,
    UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn overwrite_mode_updates_multiplier_and_costs() {
    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", 1),
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
        &snapshot("plus", 1),
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
        &snapshot("other", 1),
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
async fn model_fetch_failure_and_removed_model_are_distinct_statuses() {
    let unavailable = key_outcome(
        &failing_importer(),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", 1),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;
    let removed = key_outcome(
        &importer(&[]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", 1),
        &globals(Some(Decimal::ONE)),
        &bindings(),
        &key(Decimal::ONE),
    )
    .await;

    assert_eq!(unavailable.statuses, vec![ProviderQuickImportSyncStatus::UpstreamKeyUnavailable]);
    assert_eq!(removed.statuses, vec![ProviderQuickImportSyncStatus::UpstreamModelRemoved]);
}

#[tokio::test]
async fn missing_default_cost_is_reported() {
    let outcome = key_outcome(
        &importer(&["upstream-gpt"]),
        &source(sync_config()),
        &source_config(),
        &snapshot("plus", 1),
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
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        base_url: "https://newapi.example".into(),
        encrypted_system_access_token: "enc".into(),
        user_id: "737".into(),
        recharge_multiplier: Decimal::new(10, 0),
        sync_config,
        last_status: None,
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
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
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::ONE,
        effective_cost_multiplier,
        statuses: vec![ProviderQuickImportSyncStatus::Ok],
        model_mappings: vec![ProviderQuickImportSyncKeyModel {
            upstream_model_id: "upstream-gpt".into(),
            global_model_id: "global-gpt".into(),
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

fn snapshot(group: &str, status: i32) -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([("plus".into(), Decimal::new(2, 0)), ("other".into(), Decimal::ONE)]),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status,
            group: Some(group.into()),
        }],
    }
}

fn empty_snapshot() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([("plus".into(), Decimal::new(2, 0))]),
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
            status: 1,
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
        provider_count: None,
        active_provider_count: None,
        usage_count: None,
        created_at: String::new(),
        updated_at: None,
    }
}

fn bindings() -> BTreeMap<String, BindingInfo> {
    BTreeMap::from([(
        "global-gpt".into(),
        BindingInfo {
            id: "provider-model-1".into(),
            upstream_model_id: "upstream-gpt".into(),
        },
    )])
}
