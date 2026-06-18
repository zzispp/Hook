use std::collections::BTreeSet;

use types::provider::{
    ProviderApiKey, ProviderOrigin, ProviderQuickImportBindCommitRequest, ProviderQuickImportBindCommitResponse, ProviderQuickImportBindLocalKey,
    ProviderQuickImportBindPreviewRequest, ProviderQuickImportBindPreviewResponse, ProviderQuickImportPreviewResponse, ProviderQuickImportProviderConfig,
};

use crate::application::{GlobalModelCatalog, ProviderError, ProviderRepository, ProviderResult, SecretCipher, UpstreamProviderImportSource};

use super::{
    quick_import::QuickImportArgs,
    quick_import_commit::{QuickImportBindDraft, quick_import_bind, resolved_mappings, selected_bind_tokens},
    quick_import_preview::preview_response,
};

pub async fn preview_quick_import_bind<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    input: ProviderQuickImportBindPreviewRequest,
) -> ProviderResult<ProviderQuickImportBindPreviewResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    validate_bind_source(input.source_kind.clone(), &input.source, input.recharge_multiplier)?;
    let provider = bind_provider(args.repository, provider_id).await?;
    let local_keys = args.repository.list_api_keys(provider_id).await?;
    let data = args.importer.fetch_import_data(&input.source).await?;
    let globals = args.models.list_global_models().await?;
    Ok(ProviderQuickImportBindPreviewResponse {
        preview: bind_preview(input, data, &globals, &provider.name),
        provider,
        local_keys: local_keys.into_iter().map(local_key).collect(),
    })
}

pub async fn commit_quick_import_bind<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    provider_id: &str,
    input: ProviderQuickImportBindCommitRequest,
) -> ProviderResult<ProviderQuickImportBindCommitResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    validate_bind_source(input.source_kind.clone(), &input.source, input.recharge_multiplier)?;
    let provider = bind_provider(args.repository, provider_id).await?;
    let local_keys = args.repository.list_api_keys(provider_id).await?;
    validate_local_key_selection(&local_keys, &input)?;
    let data = args.importer.fetch_import_data(&input.source).await?;
    let globals = args.models.list_global_models().await?;
    let selected = selected_bind_tokens(&data, &input.selected_tokens)?;
    let mappings = resolved_mappings(&selected, &globals, input.selected_model_ids, input.model_mappings)?;
    let provider_config = ProviderQuickImportProviderConfig::default();
    let draft = quick_import_bind(QuickImportBindDraft {
        provider_id: provider.id.clone(),
        source: &input.source,
        provider_config: &provider_config,
        recharge_multiplier: input.recharge_multiplier,
        sync_config: input.sync_config,
        selected,
        globals: &globals,
        mappings,
        cipher: args.cipher,
    })?;
    let output = args.repository.bind_quick_import(draft).await?;
    Ok(ProviderQuickImportBindCommitResponse {
        bound_token_count: output.api_keys.len(),
        created_key_count: output.created_key_count,
        reused_key_count: output.reused_key_count,
        deleted_key_count: output.deleted_key_count,
        provider: output.provider,
        endpoints: output.endpoints,
        api_keys: output.api_keys,
        model_bindings: output.model_bindings,
        model_costs: output.model_costs,
    })
}

async fn bind_provider<R>(repository: &R, provider_id: &str) -> ProviderResult<types::provider::Provider>
where
    R: ProviderRepository,
{
    let provider = repository.find_provider(provider_id).await?.ok_or(ProviderError::NotFound)?;
    ensure_bind_provider(&provider)?;
    Ok(provider)
}

fn ensure_bind_provider(provider: &types::provider::Provider) -> ProviderResult<()> {
    if provider.provider_origin == ProviderOrigin::QuickImport {
        return Err(ProviderError::InvalidInput("provider is already a quick import provider".into()));
    }
    Ok(())
}

fn bind_preview(
    input: ProviderQuickImportBindPreviewRequest,
    data: crate::application::UpstreamImportData,
    globals: &[types::model::GlobalModelResponse],
    provider_name: &str,
) -> ProviderQuickImportPreviewResponse {
    preview_response(
        types::provider::ProviderQuickImportPreviewRequest {
            source_kind: input.source_kind,
            source: input.source,
            provider_name: provider_name.to_owned(),
            provider_config: Default::default(),
            recharge_multiplier: input.recharge_multiplier,
        },
        data,
        globals,
    )
}

fn local_key(key: ProviderApiKey) -> ProviderQuickImportBindLocalKey {
    ProviderQuickImportBindLocalKey {
        id: key.id,
        name: key.name,
        api_formats: key.api_formats,
        allowed_model_ids: key.allowed_model_ids,
        is_active: key.is_active,
    }
}

fn validate_bind_source(
    source_kind: types::provider::ProviderQuickImportSourceKind,
    source: &types::provider::ProviderQuickImportSourceConfig,
    recharge_multiplier: rust_decimal::Decimal,
) -> ProviderResult<()> {
    if source_kind != source.kind() {
        return Err(ProviderError::InvalidInput("source_kind does not match source.kind".into()));
    }
    if recharge_multiplier <= rust_decimal::Decimal::ZERO {
        return Err(ProviderError::InvalidInput("recharge_multiplier must be greater than 0".into()));
    }
    let types::provider::ProviderQuickImportSourceConfig::Newapi(config) = source;
    if config.base_url.trim().is_empty() || config.system_access_token.trim().is_empty() || config.user_id.trim().is_empty() {
        return Err(ProviderError::InvalidInput("newapi source fields cannot be blank".into()));
    }
    Ok(())
}

fn validate_local_key_selection(local_keys: &[ProviderApiKey], input: &ProviderQuickImportBindCommitRequest) -> ProviderResult<()> {
    let local_key_ids = local_keys.iter().map(|key| key.id.as_str()).collect::<BTreeSet<_>>();
    let mut selected_key_ids = BTreeSet::new();
    let mut selected_token_ids = BTreeSet::new();
    for token in &input.selected_tokens {
        let upstream_id = token.upstream_token_id.trim();
        if !selected_token_ids.insert(upstream_id.to_owned()) {
            return Err(ProviderError::InvalidInput(format!("upstream token is selected more than once: {upstream_id}")));
        }
        let Some(local_key_id) = token.local_key_id.as_deref().map(str::trim).filter(|id| !id.is_empty()) else {
            continue;
        };
        if !local_key_ids.contains(local_key_id) {
            return Err(ProviderError::InvalidInput(format!("local key does not belong to provider: {local_key_id}")));
        }
        if !selected_key_ids.insert(local_key_id.to_owned()) {
            return Err(ProviderError::InvalidInput(format!("local key is selected more than once: {local_key_id}")));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::provider::{
        Provider, ProviderApiKey, ProviderOrigin, ProviderQuickImportBindCommitRequest, ProviderQuickImportBindSelectedToken, ProviderQuickImportSourceConfig,
        ProviderQuickImportSourceKind, ProviderQuickImportSyncConfig,
    };

    use super::{ensure_bind_provider, validate_local_key_selection};

    #[test]
    fn local_key_selection_rejects_duplicate_key() {
        let local_keys = vec![local_key("key-a", "provider-a")];
        let input = bind_request(vec![selected("token-a", Some("key-a")), selected("token-b", Some("key-a"))]);

        let error = validate_local_key_selection(&local_keys, &input).unwrap_err();

        assert!(error.to_string().contains("local key is selected more than once: key-a"));
    }

    #[test]
    fn local_key_selection_rejects_key_outside_provider() {
        let local_keys = vec![local_key("key-a", "provider-a")];
        let input = bind_request(vec![selected("token-a", Some("key-b"))]);

        let error = validate_local_key_selection(&local_keys, &input).unwrap_err();

        assert!(error.to_string().contains("local key does not belong to provider: key-b"));
    }

    #[test]
    fn local_key_selection_rejects_duplicate_upstream_token() {
        let local_keys = vec![local_key("key-a", "provider-a"), local_key("key-b", "provider-a")];
        let input = bind_request(vec![selected("token-a", Some("key-a")), selected("token-a", Some("key-b"))]);

        let error = validate_local_key_selection(&local_keys, &input).unwrap_err();

        assert!(error.to_string().contains("upstream token is selected more than once: token-a"));
    }

    #[test]
    fn local_key_selection_accepts_new_and_existing_keys() {
        let local_keys = vec![local_key("key-a", "provider-a")];
        let input = bind_request(vec![selected("token-a", Some("key-a")), selected("token-b", None)]);

        validate_local_key_selection(&local_keys, &input).unwrap();
    }

    fn bind_request(selected_tokens: Vec<ProviderQuickImportBindSelectedToken>) -> ProviderQuickImportBindCommitRequest {
        ProviderQuickImportBindCommitRequest {
            source_kind: ProviderQuickImportSourceKind::Newapi,
            source: ProviderQuickImportSourceConfig::Newapi(types::provider::NewApiQuickImportConfig {
                base_url: "https://newapi.example".into(),
                system_access_token: "system-token".into(),
                user_id: "737".into(),
            }),
            recharge_multiplier: Decimal::ONE,
            selected_tokens,
            selected_model_ids: vec!["gpt-5".into()],
            model_mappings: vec![],
            sync_config: ProviderQuickImportSyncConfig::default(),
        }
    }

    fn selected(upstream_token_id: &str, local_key_id: Option<&str>) -> ProviderQuickImportBindSelectedToken {
        ProviderQuickImportBindSelectedToken {
            upstream_token_id: upstream_token_id.into(),
            local_key_id: local_key_id.map(str::to_owned),
            name: "codex".into(),
            endpoint_formats: vec!["openai".into()],
            effective_cost_multiplier: Decimal::ONE,
        }
    }

    fn local_key(id: &str, provider_id: &str) -> ProviderApiKey {
        ProviderApiKey {
            id: id.into(),
            provider_id: provider_id.into(),
            name: id.into(),
            api_formats: vec!["openai".into()],
            allowed_model_ids: vec!["global-model-a".into()],
            note: None,
            internal_priority: 10,
            global_priority_by_format: std::collections::BTreeMap::new(),
            rpm_limit: None,
            learned_rpm_limit: None,
            cache_ttl_minutes: 5,
            max_probe_interval_minutes: 32,
            time_range_enabled: false,
            time_range_start: None,
            time_range_end: None,
            health_by_format: None,
            circuit_breaker_by_format: None,
            is_active: true,
            has_api_key: true,
            quick_import_sync: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn bind_provider_rejects_quick_import_provider() {
        let provider = provider(ProviderOrigin::QuickImport);

        let error = ensure_bind_provider(&provider).unwrap_err();

        assert!(error.to_string().contains("provider is already a quick import provider"));
    }

    fn provider(origin: ProviderOrigin) -> Provider {
        Provider {
            id: "provider-a".into(),
            name: "Provider A".into(),
            provider_type: "custom".into(),
            provider_origin: origin,
            max_retries: Some(2),
            request_timeout_seconds: Some(300.0),
            stream_first_byte_timeout_seconds: Some(60.0),
            stream_idle_timeout_seconds: Some(300.0),
            priority: 100,
            keep_priority_on_conversion: false,
            enable_format_conversion: true,
            is_active: true,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
