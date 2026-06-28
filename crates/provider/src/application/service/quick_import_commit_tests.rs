use std::collections::BTreeMap;

use rust_decimal::Decimal;
use types::model::{GlobalModelResponse, TieredPricingConfig};
use types::provider::{
    NewApiQuickImportConfig, ProviderQuickImportBindSelectedToken, ProviderQuickImportModelMappingInput, ProviderQuickImportProviderConfig,
    ProviderQuickImportSelectedToken, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig,
};

use super::{
    quick_import_commit::{QuickImportBindDraft, QuickImportCreateDraft, SelectedToken, quick_import_bind, quick_import_create, resolved_mappings},
    quick_import_commit_models::assert_no_mapping_conflicts,
    quick_import_shared::provider_create,
};
use crate::application::{ProviderError, ProviderResult, SecretCipher, UpstreamImportModel, UpstreamImportToken};

#[test]
fn resolved_mappings_uses_exact_global_model_name() {
    let token = token_with_model("gpt-5");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test(&token, vec!["openai:chat".into()]),
        &[("gpt-5", "global-1")],
    )];
    let mappings = resolved_mappings(selected, &[global_model("global-1", "gpt-5")]).unwrap();

    assert_eq!(mappings[0].resolved_mappings.get("gpt-5"), Some(&"global-1".to_owned()));
}

#[test]
fn mapping_conflict_is_rejected() {
    let mappings = BTreeMap::from([("upstream-a".into(), "global-1".into()), ("upstream-b".into(), "global-1".into())]);

    assert!(assert_no_mapping_conflicts(&mappings).is_err());
}

#[test]
fn missing_mapping_is_rejected() {
    let token = token_with_model("upstream-only");
    let selected = vec![SelectedToken::for_test(&token, vec!["openai".into()])];

    let error = resolved_mappings(selected, &[global_model("global-1", "gpt-5")]).unwrap_err();

    assert!(error.to_string().contains("selected token has no model mappings: 1209"));
}

#[test]
fn unselected_mapping_is_rejected() {
    let token = token_with_models(&["upstream-only", "other-upstream"]);
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test(&token, vec!["openai".into()]),
        &[("missing-upstream", "global-2")],
    )];
    let globals = [global_model("global-1", "upstream-only"), global_model("global-2", "other")];
    let error = resolved_mappings(selected, &globals).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("selected model does not exist on selected token 1209: missing-upstream")
    );
}

#[test]
fn selected_token_rejects_non_positive_effective_multiplier() {
    let data = crate::application::UpstreamImportData {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        tokens: vec![token_with_model("gpt-5")],
    };
    let inputs = vec![ProviderQuickImportSelectedToken {
        upstream_token_id: "1209".into(),
        name: "codex".into(),
        endpoint_formats: vec!["openai".into()],
        effective_cost_multiplier: Decimal::ZERO,
        model_mappings: vec![mapping_input("gpt-5", "global-1")],
    }];

    let error = match super::quick_import_commit::selected_tokens(&data, &inputs) {
        Ok(_) => panic!("selected token should reject non-positive effective multiplier"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("effective_cost_multiplier must be greater than 0"));
}

#[test]
fn quick_import_create_builds_complete_resource_set() {
    let token = token_with_model("upstream-gpt-5");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test_with_multiplier(&token, vec!["openai".into(), "codex".into()], Decimal::new(1, 1)),
        &[("upstream-gpt-5", "global-1")],
    )];
    let globals = [global_model("global-1", "gpt-5")];

    let draft = quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &provider_config()),
        provider_config: &provider_config(),
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.provider.name, "Provider A");
    assert_eq!(draft.sync_source.as_ref().unwrap().recharge_multiplier, Decimal::ONE);
    assert!(draft.sync_source.as_ref().unwrap().sync_config.auto_sync_enabled);
    assert_eq!(draft.endpoints.len(), 2);
    assert_eq!(draft.model_bindings[0].global_model_id, "global-1");
    assert_eq!(draft.api_keys[0].encrypted_api_key, "enc:sk-test");
    assert_eq!(draft.api_keys[0].input.allowed_model_ids, vec!["global-1"]);
    assert_eq!(draft.api_keys[0].model_mappings[0].upstream_model_name, "upstream-gpt-5");
    assert_eq!(draft.model_costs[0].cost.price_per_request, Some(Decimal::new(1, 1)));
}

#[test]
fn quick_import_bind_builds_bound_resource_set() {
    let token = token_with_model("upstream-gpt-5");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test_with_local_key(&token, "key-existing", vec!["openai".into()]),
        &[("upstream-gpt-5", "global-1")],
    )];
    let globals = [global_model("global-1", "gpt-5")];

    let draft = quick_import_bind(QuickImportBindDraft {
        provider_id: "provider-a".into(),
        source: &source_config(),
        provider_config: &provider_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.provider_id, "provider-a");
    assert_eq!(draft.sync_source.encrypted_system_access_token, "enc:system-token");
    assert_eq!(draft.api_keys[0].local_key_id.as_deref(), Some("key-existing"));
    assert_eq!(draft.api_keys[0].create.encrypted_api_key, "enc:sk-test");
    assert_eq!(draft.api_keys[0].create.input.allowed_model_ids, vec!["global-1"]);
    assert_eq!(draft.model_costs[0].upstream_token_id, "1209");
}

#[test]
fn selected_bind_token_keeps_local_key_id() {
    let data = crate::application::UpstreamImportData {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        tokens: vec![token_with_model("gpt-5")],
    };
    let inputs = vec![ProviderQuickImportBindSelectedToken {
        upstream_token_id: "1209".into(),
        local_key_id: Some(" key-a ".into()),
        name: "codex".into(),
        endpoint_formats: vec!["openai".into()],
        effective_cost_multiplier: Decimal::ONE,
        model_mappings: vec![mapping_input("gpt-5", "global-1")],
    }];

    let selected = super::quick_import_commit::selected_bind_tokens(&data, &inputs).unwrap();

    assert_eq!(selected[0].local_key_id.as_deref(), Some("key-a"));
}

#[test]
fn selected_bind_token_rejects_disabled_upstream_token() {
    let mut token = token_with_model("gpt-5");
    token.status = "disabled".into();
    token.is_active = false;
    let data = crate::application::UpstreamImportData {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        tokens: vec![token],
    };
    let inputs = vec![bind_token_input("1209")];

    let error = super::quick_import_commit::selected_bind_tokens(&data, &inputs).unwrap_err();

    assert!(error.to_string().contains("upstream token is disabled: 1209"));
}

#[test]
fn selected_bind_token_rejects_missing_upstream_key_during_draft_build() {
    let mut token = token_with_model("gpt-5");
    token.api_key = None;
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test_with_local_key(&token, "key-a", vec!["openai".into()]),
        &[("gpt-5", "global-1")],
    )];
    let globals = [global_model("global-1", "gpt-5")];

    let error = quick_import_bind(QuickImportBindDraft {
        provider_id: "provider-a".into(),
        source: &source_config(),
        provider_config: &provider_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap_err();

    assert!(error.to_string().contains("newapi key was not fetched for selected token: 1209"));
}

#[test]
fn selected_bind_token_rejects_missing_mapping() {
    let token = token_with_model("upstream-only");
    let selected = vec![SelectedToken::for_test_with_local_key(&token, "key-a", vec!["openai".into()])];

    let error = resolved_mappings(selected, &[global_model("global-1", "gpt-5")]).unwrap_err();

    assert!(error.to_string().contains("selected token has no model mappings: 1209"));
}

#[test]
fn quick_import_bind_rejects_missing_cost() {
    let token = token_with_model("gpt-5");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test_with_local_key(&token, "key-a", vec!["openai".into()]),
        &[("gpt-5", "global-1")],
    )];
    let globals = [global_model_without_cost("global-1", "gpt-5")];

    let error = quick_import_bind(QuickImportBindDraft {
        provider_id: "provider-a".into(),
        source: &source_config(),
        provider_config: &provider_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap_err();

    assert!(error.to_string().contains("global model has no default cost"));
}

#[test]
fn selected_token_name_is_used_for_imported_key() {
    let data = crate::application::UpstreamImportData {
        source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
        tokens: vec![token_with_model("gpt-5")],
    };
    let inputs = vec![ProviderQuickImportSelectedToken {
        upstream_token_id: "1209".into(),
        name: "custom key name".into(),
        endpoint_formats: vec!["openai:chat".into()],
        effective_cost_multiplier: Decimal::ONE,
        model_mappings: vec![mapping_input("gpt-5", "global-1")],
    }];
    let selected = resolved_mappings(
        super::quick_import_commit::selected_tokens(&data, &inputs).unwrap(),
        &[global_model("global-1", "gpt-5")],
    )
    .unwrap();
    let globals = [global_model("global-1", "gpt-5")];

    let draft = quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &provider_config()),
        provider_config: &provider_config(),
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.api_keys[0].input.name, "custom key name");
}

#[test]
fn quick_import_create_does_not_write_mapping_for_same_model_name() {
    let token = token_with_model("gpt-5");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test(&token, vec!["openai".into()]),
        &[("gpt-5", "global-1")],
    )];
    let globals = [global_model("global-1", "gpt-5")];

    let draft = quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &provider_config()),
        provider_config: &provider_config(),
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.model_bindings[0].global_model_id, "global-1");
}

#[test]
fn quick_import_create_imports_only_mapped_token_models() {
    let token = token_with_models(&["upstream-gpt-5", "upstream-gpt-image"]);
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test_with_multiplier(&token, vec!["openai".into()], Decimal::new(1, 1)),
        &[("upstream-gpt-5", "global-1")],
    )];
    let globals = [global_model("global-1", "gpt-5"), global_model("global-2", "gpt-image")];

    let draft = quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &provider_config()),
        provider_config: &provider_config(),
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.model_bindings.len(), 1);
    assert_eq!(draft.api_keys[0].input.allowed_model_ids, vec!["global-1"]);
    assert_eq!(draft.model_costs.len(), 1);
    assert_eq!(draft.model_costs[0].global_model_id, "global-1");
}

#[test]
fn quick_import_create_keeps_same_upstream_model_independent_per_token() {
    let token_a = token_with_id_and_models("token-a", &["shared-model"]);
    let token_b = token_with_id_and_models("token-b", &["shared-model"]);
    let selected = vec![
        with_resolved_mappings(SelectedToken::for_test(&token_a, vec!["openai".into()]), &[("shared-model", "global-1")]),
        with_resolved_mappings(SelectedToken::for_test(&token_b, vec!["openai".into()]), &[("shared-model", "global-2")]),
    ];
    let globals = [global_model("global-1", "global-a"), global_model("global-2", "global-b")];

    let draft = quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &provider_config()),
        provider_config: &provider_config(),
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap();

    assert_eq!(draft.model_bindings.len(), 2);
    assert_eq!(draft.api_keys.len(), 2);
    assert_eq!(draft.api_keys[0].input.allowed_model_ids, vec!["global-1"]);
    assert_eq!(draft.api_keys[1].input.allowed_model_ids, vec!["global-2"]);
    assert_eq!(draft.api_keys[0].model_mappings[0].global_model_id, "global-1");
    assert_eq!(draft.api_keys[1].model_mappings[0].global_model_id, "global-2");
}

#[test]
fn provider_create_uses_quick_import_provider_config() {
    let config = ProviderQuickImportProviderConfig {
        max_retries: Some(5),
        request_timeout_seconds: Some(120.0),
        stream_first_byte_timeout_seconds: Some(30.0),
        stream_first_output_timeout_seconds: Some(45.0),
        stream_idle_timeout_seconds: Some(90.0),
        priority: Some(80),
        keep_priority_on_conversion: Some(true),
        enable_format_conversion: Some(false),
        is_active: Some(false),
        upstream_image_native_stream: Some(false),
    };

    let provider = provider_create(" Provider A ", &config);

    assert_eq!(provider.name, "Provider A");
    assert_eq!(provider.max_retries, Some(5));
    assert_eq!(provider.request_timeout_seconds, Some(120.0));
    assert_eq!(provider.stream_first_byte_timeout_seconds, Some(30.0));
    assert_eq!(provider.stream_first_output_timeout_seconds, Some(45.0));
    assert_eq!(provider.stream_idle_timeout_seconds, Some(90.0));
    assert_eq!(provider.priority, Some(80));
    assert_eq!(provider.keep_priority_on_conversion, Some(true));
    assert_eq!(provider.enable_format_conversion, Some(false));
    assert_eq!(provider.is_active, Some(false));
}

#[test]
fn quick_import_image_endpoint_defaults_to_sync_wrapped_stream() {
    let draft = quick_import_create_image_endpoint(provider_config());

    assert_eq!(
        draft.endpoints[0].format_acceptance_config,
        Some(serde_json::json!({"upstream_image_stream_mode": "sync_wrapped_stream"}))
    );
}

#[test]
fn quick_import_image_endpoint_can_enable_native_stream() {
    let config = ProviderQuickImportProviderConfig {
        upstream_image_native_stream: Some(true),
        ..ProviderQuickImportProviderConfig::default()
    };
    let draft = quick_import_create_image_endpoint(config);

    assert_eq!(
        draft.endpoints[0].format_acceptance_config,
        Some(serde_json::json!({"upstream_image_stream_mode": "native_stream"}))
    );
}

fn quick_import_create_image_endpoint(config: ProviderQuickImportProviderConfig) -> crate::application::ProviderQuickImportCreate {
    let token = token_with_model("gpt-image-1");
    let selected = vec![with_resolved_mappings(
        SelectedToken::for_test(&token, vec!["openai_image".into()]),
        &[("gpt-image-1", "global-image")],
    )];
    let globals = [global_model("global-image", "gpt-image-1")];

    quick_import_create(QuickImportCreateDraft {
        provider: provider_create("Provider A", &config),
        provider_config: &config,
        source: &source_config(),
        recharge_multiplier: Decimal::ONE,
        sync_config: ProviderQuickImportSyncConfig::default(),
        selected,
        globals: &globals,
        cipher: &TestCipher,
    })
    .unwrap()
}

fn token_with_model(model: &str) -> UpstreamImportToken {
    token_with_models(&[model])
}

fn token_with_models(models: &[&str]) -> UpstreamImportToken {
    token_with_id_and_models("1209", models)
}

fn token_with_id_and_models(id: &str, models: &[&str]) -> UpstreamImportToken {
    UpstreamImportToken {
        id: id.into(),
        name: "codex".into(),
        masked_key: "abcd****efgh".into(),
        status: "active".into(),
        is_active: true,
        group: Some("plus".into()),
        group_ratio: Decimal::ONE,
        api_key: Some("sk-test".into()),
        models: models
            .iter()
            .map(|model| UpstreamImportModel {
                id: (*model).into(),
                supported_endpoint_types: vec![],
            })
            .collect(),
    }
}

fn mapping_input(upstream_model_id: &str, global_model_id: &str) -> ProviderQuickImportModelMappingInput {
    ProviderQuickImportModelMappingInput {
        upstream_model_id: upstream_model_id.into(),
        global_model_id: global_model_id.into(),
    }
}

fn with_resolved_mappings<'a>(mut selected: SelectedToken<'a>, mappings: &[(&str, &str)]) -> SelectedToken<'a> {
    selected.resolved_mappings = mappings
        .iter()
        .map(|(upstream_model_id, global_model_id)| ((*upstream_model_id).to_owned(), (*global_model_id).to_owned()))
        .collect();
    selected
}

fn bind_token_input(upstream_token_id: &str) -> ProviderQuickImportBindSelectedToken {
    ProviderQuickImportBindSelectedToken {
        upstream_token_id: upstream_token_id.into(),
        local_key_id: Some("key-a".into()),
        name: "codex".into(),
        endpoint_formats: vec!["openai".into()],
        effective_cost_multiplier: Decimal::ONE,
        model_mappings: vec![mapping_input("gpt-5", "global-1")],
    }
}

fn source_config() -> ProviderQuickImportSourceConfig {
    ProviderQuickImportSourceConfig::Newapi(NewApiQuickImportConfig {
        base_url: "https://newapi.example/".into(),
        system_access_token: "system-token".into(),
        user_id: "737".into(),
    })
}

fn provider_config() -> ProviderQuickImportProviderConfig {
    ProviderQuickImportProviderConfig::default()
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

fn global_model_without_cost(id: &str, name: &str) -> GlobalModelResponse {
    GlobalModelResponse {
        default_price_per_request: None,
        default_tiered_pricing: TieredPricingConfig { tiers: vec![] },
        ..global_model(id, name)
    }
}

struct TestCipher;

impl SecretCipher for TestCipher {
    fn encrypt_provider_key(&self, plaintext: &str) -> ProviderResult<String> {
        Ok(format!("enc:{plaintext}"))
    }

    fn decrypt_provider_key(&self, _ciphertext: &str) -> ProviderResult<String> {
        Err(ProviderError::Infrastructure("not used".into()))
    }
}
