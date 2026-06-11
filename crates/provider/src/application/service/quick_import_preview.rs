use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderQuickImportCostIssue, ProviderQuickImportLinkedKeyPreview, ProviderQuickImportModelMappingPreview, ProviderQuickImportPreviewRequest,
        ProviderQuickImportPreviewResponse, ProviderQuickImportRemoteModel, ProviderQuickImportSourceKind, ProviderQuickImportTokenPreview,
    },
};

use crate::application::{UpstreamImportData, UpstreamImportToken};

use super::{quick_import_costs::has_default_cost, quick_import_shared::globals_by_name};

pub fn preview_response(
    input: ProviderQuickImportPreviewRequest,
    data: UpstreamImportData,
    globals: &[GlobalModelResponse],
) -> ProviderQuickImportPreviewResponse {
    let by_name = globals_by_name(globals);
    ProviderQuickImportPreviewResponse {
        provider_id: None,
        source_kind: data.source_kind.clone(),
        provider_name: input.provider_name.trim().to_owned(),
        recharge_multiplier: input.recharge_multiplier,
        tokens: token_previews(&data.tokens, input.recharge_multiplier, &by_name),
        model_mappings: mapping_previews(&data, &by_name),
    }
}

pub struct AppendPreviewInput<'a> {
    pub provider_id: String,
    pub provider_name: String,
    pub source_kind: ProviderQuickImportSourceKind,
    pub recharge_multiplier: Decimal,
    pub data: UpstreamImportData,
    pub globals: &'a [GlobalModelResponse],
    pub imported_token_ids: &'a BTreeSet<String>,
    pub linked_keys: &'a BTreeMap<String, ProviderQuickImportLinkedKeyPreview>,
    pub include_linked_tokens: bool,
}

pub fn append_preview_response(input: AppendPreviewInput<'_>) -> ProviderQuickImportPreviewResponse {
    let by_name = globals_by_name(input.globals);
    let visible_tokens = input
        .data
        .tokens
        .iter()
        .filter(|token| input.include_linked_tokens || !input.imported_token_ids.contains(&token.id))
        .cloned()
        .collect::<Vec<_>>();
    let tokens = visible_tokens
        .iter()
        .map(|token| {
            append_token_preview(
                token,
                input.recharge_multiplier,
                &by_name,
                input.imported_token_ids.contains(&token.id),
                input.linked_keys.get(&token.id),
            )
        })
        .collect();
    ProviderQuickImportPreviewResponse {
        provider_id: Some(input.provider_id),
        source_kind: input.source_kind,
        provider_name: input.provider_name,
        recharge_multiplier: input.recharge_multiplier,
        tokens,
        model_mappings: mapping_previews_for_tokens(&visible_tokens, &by_name),
    }
}

fn token_previews(
    tokens: &[UpstreamImportToken],
    recharge_multiplier: Decimal,
    by_name: &BTreeMap<String, &GlobalModelResponse>,
) -> Vec<ProviderQuickImportTokenPreview> {
    tokens.iter().map(|token| token_preview(token, recharge_multiplier, by_name)).collect()
}

fn token_preview(
    token: &UpstreamImportToken,
    recharge_multiplier: Decimal,
    by_name: &BTreeMap<String, &GlobalModelResponse>,
) -> ProviderQuickImportTokenPreview {
    ProviderQuickImportTokenPreview {
        upstream_token_id: token.id.clone(),
        name: token.name.clone(),
        masked_key: token.masked_key.clone(),
        status: token.status,
        group: token.group.clone(),
        group_ratio: token.group_ratio,
        effective_cost_multiplier: token.group_ratio / recharge_multiplier,
        importable: token_importable(token),
        already_imported: false,
        import_block_reason: None,
        linked_key: None,
        models: remote_models(token, by_name),
        cost_issues: cost_issues(token, by_name),
    }
}

fn append_token_preview(
    token: &UpstreamImportToken,
    recharge_multiplier: Decimal,
    by_name: &BTreeMap<String, &GlobalModelResponse>,
    already_imported: bool,
    linked_key: Option<&ProviderQuickImportLinkedKeyPreview>,
) -> ProviderQuickImportTokenPreview {
    ProviderQuickImportTokenPreview {
        already_imported,
        importable: token_importable(token) && !already_imported,
        import_block_reason: append_block_reason(token, already_imported),
        linked_key: linked_key.cloned(),
        ..token_preview(token, recharge_multiplier, by_name)
    }
}

fn append_block_reason(token: &UpstreamImportToken, already_imported: bool) -> Option<String> {
    if already_imported {
        return Some("upstream token is already linked".into());
    }
    if token.status != 1 {
        return Some("upstream token is disabled".into());
    }
    token.group.is_none().then(|| "upstream token group is missing".into())
}

fn token_importable(token: &UpstreamImportToken) -> bool {
    token.status == 1 && token.group.is_some()
}

fn remote_models(token: &UpstreamImportToken, by_name: &BTreeMap<String, &GlobalModelResponse>) -> Vec<ProviderQuickImportRemoteModel> {
    token
        .models
        .iter()
        .map(|model| ProviderQuickImportRemoteModel {
            upstream_model_id: model.id.clone(),
            suggested_global_model_id: by_name.get(&model.id).map(|global| global.id.clone()),
            supported_endpoint_types: model.supported_endpoint_types.clone(),
        })
        .collect()
}

fn cost_issues(token: &UpstreamImportToken, by_name: &BTreeMap<String, &GlobalModelResponse>) -> Vec<ProviderQuickImportCostIssue> {
    token
        .models
        .iter()
        .filter_map(|model| preview_cost_issue(&model.id, by_name.get(&model.id).copied()))
        .collect()
}

fn preview_cost_issue(upstream_model_id: &str, global: Option<&GlobalModelResponse>) -> Option<ProviderQuickImportCostIssue> {
    let Some(global) = global else {
        return Some(cost_issue(upstream_model_id, None, "model mapping is required"));
    };
    (!has_default_cost(global)).then(|| cost_issue(upstream_model_id, Some(&global.id), "global model has no default cost"))
}

fn mapping_previews(data: &UpstreamImportData, by_name: &BTreeMap<String, &GlobalModelResponse>) -> Vec<ProviderQuickImportModelMappingPreview> {
    mapping_previews_for_tokens(&data.tokens, by_name)
}

fn mapping_previews_for_tokens(
    tokens: &[UpstreamImportToken],
    by_name: &BTreeMap<String, &GlobalModelResponse>,
) -> Vec<ProviderQuickImportModelMappingPreview> {
    upstream_model_ids(tokens)
        .into_iter()
        .map(|id| {
            let suggested = by_name.get(&id).map(|model| model.id.clone());
            ProviderQuickImportModelMappingPreview {
                upstream_model_id: id,
                required: suggested.is_none(),
                suggested_global_model_id: suggested,
            }
        })
        .collect()
}

fn upstream_model_ids(tokens: &[UpstreamImportToken]) -> BTreeSet<String> {
    tokens.iter().flat_map(|token| token.models.iter().map(|model| model.id.clone())).collect()
}

fn cost_issue(upstream_model_id: &str, global_model_id: Option<&str>, message: &str) -> ProviderQuickImportCostIssue {
    ProviderQuickImportCostIssue {
        upstream_model_id: upstream_model_id.to_owned(),
        global_model_id: global_model_id.map(str::to_owned),
        message: message.to_owned(),
    }
}
