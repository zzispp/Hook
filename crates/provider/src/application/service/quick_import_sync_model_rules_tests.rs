use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, TieredPricingConfig},
    provider::{NewApiQuickImportConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus},
};

use super::{
    quick_import_sync_bindings::BindingInfo, quick_import_sync_candidates::candidate_model_ids, quick_import_sync_events::key_events,
    quick_import_sync_outcome::key_outcome,
};
use crate::application::{
    ProviderQuickImportSyncKey, ProviderQuickImportSyncSource, ProviderResult, UpstreamGroupRatio, UpstreamImportData, UpstreamImportModel,
    UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn key_without_associated_models_is_disabled() {
    let outcome = key_outcome(&TestImporter, &source(), &source_config(), &snapshot(), &globals(), &bindings(), &key(vec![])).await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::NoAssociatedModels]);
    assert!(outcome.disable_key);
}

#[tokio::test]
async fn candidate_model_status_is_warning_only() {
    let sync_source = source();
    let sync_key = key(vec![("upstream-gpt".into(), "global-gpt".into())]);
    let outcome = key_outcome(&TestImporter, &sync_source, &source_config(), &snapshot(), &globals(), &bindings(), &sync_key).await;
    let events = key_events(&sync_source, &sync_key, &outcome);

    assert_eq!(
        outcome.statuses,
        vec![ProviderQuickImportSyncStatus::Ok, ProviderQuickImportSyncStatus::ModelCandidateAvailable]
    );
    assert!(!outcome.disable_key);
    assert!(
        events
            .iter()
            .any(|event| event.status == ProviderQuickImportSyncStatus::ModelCandidateAvailable)
    );
}

#[test]
fn exact_name_upstream_model_is_reported_as_candidate_only() {
    let candidates = candidate_model_ids(
        &globals(),
        &bindings(),
        &key(vec![("upstream-gpt".into(), "global-gpt".into())]),
        &[
            UpstreamImportModel {
                id: "upstream-gpt".into(),
                supported_endpoint_types: vec![],
            },
            UpstreamImportModel {
                id: "new-global-model".into(),
                supported_endpoint_types: vec![],
            },
        ],
    );

    assert_eq!(candidates, vec!["new-global-model".to_owned()]);
}

struct TestImporter;

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("model rule tests provide snapshots directly")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("model rule tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        Ok(vec![
            UpstreamImportModel {
                id: "upstream-gpt".into(),
                supported_endpoint_types: vec![],
            },
            UpstreamImportModel {
                id: "new-global-model".into(),
                supported_endpoint_types: vec![],
            },
        ])
    }
}

fn source() -> ProviderQuickImportSyncSource {
    ProviderQuickImportSyncSource {
        id: "source-1".into(),
        provider_id: "provider-1".into(),
        provider_name: "OpenAI".into(),
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        base_url: "https://newapi.example".into(),
        encrypted_system_access_token: "enc".into(),
        user_id: "737".into(),
        recharge_multiplier: Decimal::new(10, 0),
        sync_config: ProviderQuickImportSyncConfig::default(),
        last_status: None,
        last_error: None,
        last_synced_at: None,
        consecutive_failures: 0,
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
        groups: BTreeMap::from([("plus".into(), UpstreamGroupRatio::Fixed(Decimal::new(2, 0)))]),
        tokens: vec![UpstreamSyncToken {
            id: "1209".into(),
            name: "codex".into(),
            masked_key: "abcd****efgh".into(),
            status: 1,
            group: Some("plus".into()),
        }],
    }
}

fn key(model_mappings: Vec<(String, String)>) -> ProviderQuickImportSyncKey {
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
        model_mappings: model_mappings
            .into_iter()
            .map(|(upstream_model_id, global_model_id)| crate::application::ProviderQuickImportSyncKeyModel {
                upstream_model_id,
                global_model_id,
            })
            .collect(),
    }
}

fn globals() -> BTreeMap<String, GlobalModelResponse> {
    BTreeMap::from([
        ("global-gpt".into(), global_model("global-gpt", "gpt")),
        ("new-global-model-id".into(), global_model("new-global-model-id", "new-global-model")),
    ])
}

fn global_model(id: &str, name: &str) -> GlobalModelResponse {
    GlobalModelResponse {
        id: id.into(),
        name: name.into(),
        display_name: name.into(),
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
