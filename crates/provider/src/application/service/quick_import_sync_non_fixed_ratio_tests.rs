use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    model::{GlobalModelResponse, TieredPricingConfig},
    provider::{NewApiQuickImportConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig, ProviderQuickImportSyncStatus},
};

use super::{quick_import_sync_bindings::BindingInfo, quick_import_sync_outcome::key_outcome};
use crate::application::{
    ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, ProviderQuickImportSyncSource, ProviderResult, UpstreamGroupRatio, UpstreamImportData,
    UpstreamImportModel, UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

#[tokio::test]
async fn non_fixed_group_ratio_is_cost_unavailable() {
    let outcome = key_outcome(&TestImporter, &source(), &source_config(), &snapshot(), &globals(), &bindings(), &key()).await;

    assert_eq!(outcome.statuses, vec![ProviderQuickImportSyncStatus::CostUnavailable]);
    assert_eq!(
        outcome.patch("key-1".into()).last_error,
        Some("newapi group ratio is not fixed for group plus: 自动".into())
    );
}

struct TestImporter;

#[async_trait]
impl UpstreamProviderImportSource for TestImporter {
    async fn fetch_import_data(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamImportData> {
        unreachable!("non-fixed ratio tests provide snapshots directly")
    }

    async fn fetch_sync_snapshot(&self, _source: &ProviderQuickImportSourceConfig) -> ProviderResult<UpstreamSyncSnapshot> {
        unreachable!("non-fixed ratio tests provide snapshots directly")
    }

    async fn fetch_sync_token_models(&self, _source: &ProviderQuickImportSourceConfig, _upstream_token_id: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
        Ok(vec![UpstreamImportModel {
            id: "upstream-gpt".into(),
            supported_endpoint_types: vec![],
        }])
    }
}

fn snapshot() -> UpstreamSyncSnapshot {
    UpstreamSyncSnapshot {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        groups: BTreeMap::from([("plus".into(), UpstreamGroupRatio::UpstreamValue("自动".into()))]),
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

fn source() -> ProviderQuickImportSyncSource {
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

fn bindings() -> BTreeMap<String, BindingInfo> {
    BTreeMap::from([("global-gpt".into(), BindingInfo { id: "provider-model-1".into() })])
}
