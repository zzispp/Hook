use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, TieredPricingConfig},
    provider::{
        NewApiQuickImportConfig, ProviderQuickImportCostSyncMode, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus,
    },
};

use super::{quick_import_sync_bindings::BindingInfo, quick_import_sync_events::key_events, quick_import_sync_outcome::key_outcome};
use crate::application::{
    ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncSource, ProviderResult, UpstreamGroupRatio, UpstreamImportData,
    UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn overwrite_mode_suppresses_event_when_only_extra_decimal_precision_differs() {
    let source = source(sync_config());
    let key = key(stored_effective_multiplier());
    let outcome = key_outcome(&importer(), &source, &source_config(), &snapshot(), &globals(), &bindings(), &key).await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::Ok]);
    assert_eq!(outcome.observed_effective_multiplier, Some(stored_effective_multiplier()));
    assert_eq!(outcome.costs.as_ref().unwrap()[0].price_per_request, Some(stored_effective_multiplier()));
    assert_eq!(outcome.patch("key-1".into()).effective_cost_multiplier, Some(stored_effective_multiplier()));
    assert!(key_events(&source, &key, &outcome).is_empty());
}

#[tokio::test]
async fn report_only_mode_keeps_ok_when_only_extra_decimal_precision_differs() {
    let mut config = sync_config();
    config.cost_sync_mode = ProviderQuickImportCostSyncMode::ReportOnly;
    let source = source(config);
    let key = key(stored_effective_multiplier());
    let outcome = key_outcome(&importer(), &source, &source_config(), &snapshot(), &globals(), &bindings(), &key).await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::Ok]);
    assert!(outcome.costs.is_none());
    assert_eq!(outcome.patch("key-1".into()).effective_cost_multiplier, None);
    assert!(key_events(&source, &key, &outcome).is_empty());
}

fn importer() -> TestImporter {
    TestImporter
}

struct TestImporter;

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("sync precision tests do not fetch import data")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("sync precision tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        Ok(vec![UpstreamImportModel {
            id: "upstream-gpt".into(),
            supported_endpoint_types: vec![],
        }])
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
        recharge_multiplier: Decimal::new(1528, 2),
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
        local_key_name: "生产主 Key".into(),
        upstream_token_id: "1209".into(),
        upstream_token_name: "codex".into(),
        upstream_group: Some("plus".into()),
        upstream_group_ratio: Decimal::new(28, 1),
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

fn snapshot() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([("plus".into(), UpstreamGroupRatio::Fixed(Decimal::new(28, 1)))]),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status: "active".into(),
            is_active: true,
            group: Some("plus".into()),
        }],
    }
}

fn globals() -> BTreeMap<String, GlobalModelResponse> {
    BTreeMap::from([("global-gpt".into(), global_model())])
}

fn global_model() -> GlobalModelResponse {
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

fn stored_effective_multiplier() -> Decimal {
    Decimal::new(18324607, 8)
}
