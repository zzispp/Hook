use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderQuickImportCostIssue, ProviderQuickImportModelMappingPreview, ProviderQuickImportPreviewRequest, ProviderQuickImportPreviewResponse,
        ProviderQuickImportRemoteModel, ProviderQuickImportTokenPreview,
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
        source_kind: data.source_kind.clone(),
        provider_name: input.provider_name.trim().to_owned(),
        recharge_multiplier: input.recharge_multiplier,
        tokens: token_previews(&data.tokens, input.recharge_multiplier, &by_name),
        model_mappings: mapping_previews(&data, &by_name),
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
        importable: token.status == 1,
        models: remote_models(token, by_name),
        cost_issues: cost_issues(token, by_name),
    }
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
    upstream_model_ids(&data.tokens)
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
