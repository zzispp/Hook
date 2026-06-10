use types::provider::{
    ProviderQuickImportCommitRequest, ProviderQuickImportCommitResponse, ProviderQuickImportPreviewRequest, ProviderQuickImportPreviewResponse,
};

use crate::application::{GlobalModelCatalog, ProviderRepository, ProviderResult, SecretCipher, UpstreamProviderImportSource};

use super::{
    provider_core::prepare_provider_create,
    quick_import_commit::{quick_import_create, resolved_mappings, selected_tokens},
    quick_import_preview::preview_response,
    quick_import_shared::{provider_create, validate_common},
};

pub struct QuickImportArgs<'a, R, M, C, I> {
    pub repository: &'a R,
    pub models: &'a M,
    pub cipher: &'a C,
    pub importer: &'a I,
}

pub async fn preview_quick_import<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    input: ProviderQuickImportPreviewRequest,
) -> ProviderResult<ProviderQuickImportPreviewResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    validate_common(input.source_kind.clone(), &input.source, &input.provider_name, input.recharge_multiplier)?;
    prepare_provider_create(args.repository, provider_create(&input.provider_name, &input.provider_config)).await?;
    let data = args.importer.fetch_import_data(&input.source).await?;
    let globals = args.models.list_global_models().await?;
    Ok(preview_response(input, data, &globals))
}

pub async fn commit_quick_import<R, M, C, I>(
    args: QuickImportArgs<'_, R, M, C, I>,
    input: ProviderQuickImportCommitRequest,
) -> ProviderResult<ProviderQuickImportCommitResponse>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    validate_common(input.source_kind.clone(), &input.source, &input.provider_name, input.recharge_multiplier)?;
    let provider = prepare_provider_create(args.repository, provider_create(&input.provider_name, &input.provider_config)).await?;
    let data = args.importer.fetch_import_data(&input.source).await?;
    let globals = args.models.list_global_models().await?;
    let selected = selected_tokens(&data, &input.selected_tokens)?;
    let mappings = resolved_mappings(&selected, &globals, input.selected_model_ids, input.model_mappings)?;
    let draft = quick_import_create(provider, &input.source, selected, &globals, mappings, args.cipher)?;
    let output = args.repository.create_quick_import(draft).await?;
    Ok(ProviderQuickImportCommitResponse {
        imported_token_count: output.api_keys.len(),
        imported_model_count: output.model_bindings.len(),
        provider: output.provider,
        endpoints: output.endpoints,
        api_keys: output.api_keys,
        model_bindings: output.model_bindings,
        model_costs: output.model_costs,
    })
}
