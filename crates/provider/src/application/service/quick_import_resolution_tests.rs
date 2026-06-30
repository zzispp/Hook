use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::application::{ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyModel, UpstreamImportModel, UpstreamImportToken};

use super::quick_import_resolution::accept_current_mappings;

#[test]
fn accept_current_mappings_drop_missing_upstream_models() {
    let key = ProviderQuickImportSyncKey {
        provider_id: "provider-1".to_owned(),
        source_id: "source-1".to_owned(),
        key_id: "key-1".to_owned(),
        local_key_name: "token-a".to_owned(),
        upstream_token_id: "upstream-1".to_owned(),
        upstream_token_name: "token-a".to_owned(),
        upstream_group_id: None,
        upstream_group: Some("plus".to_owned()),
        upstream_group_ratio: Decimal::ONE,
        effective_cost_multiplier: Decimal::ONE,
        statuses: vec![],
        model_mappings: vec![
            ProviderQuickImportSyncKeyModel {
                provider_model_id: "provider-model-1".to_owned(),
                global_model_id: "global-1".to_owned(),
                upstream_model_name: "claude-sonnet-4-5-20251001".to_owned(),
                reasoning_effort: None,
            },
            ProviderQuickImportSyncKeyModel {
                provider_model_id: "provider-model-2".to_owned(),
                global_model_id: "global-2".to_owned(),
                upstream_model_name: "claude-haiku-4-5-20251001".to_owned(),
                reasoning_effort: None,
            },
        ],
    };
    let token = UpstreamImportToken {
        id: "upstream-1".to_owned(),
        name: "token-a".to_owned(),
        masked_key: "sk-***".to_owned(),
        status: "active".to_owned(),
        is_active: true,
        group_id: None,
        group: Some("plus".to_owned()),
        group_ratio: Decimal::ONE,
        api_key: Some("sk-test".to_owned()),
        models: vec![UpstreamImportModel {
            id: "claude-sonnet-4-5-20251001".to_owned(),
            supported_endpoint_types: vec!["anthropic".to_owned()],
        }],
    };

    let mappings = accept_current_mappings(&key, &[], &token);

    assert_eq!(mappings, BTreeMap::from([("claude-sonnet-4-5-20251001".to_owned(), "global-1".to_owned())]));
}
