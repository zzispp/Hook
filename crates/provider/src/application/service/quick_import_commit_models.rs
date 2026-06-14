use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{ProviderQuickImportBindSelectedToken, ProviderQuickImportModelMappingInput, ProviderQuickImportSelectedToken},
};

use crate::application::{ProviderError, ProviderQuickImportKeyModelCreate, ProviderResult, UpstreamImportData, UpstreamImportToken};

use super::quick_import_shared::{global_model, globals_by_id, globals_by_name};

#[derive(Debug)]
pub(super) struct SelectedToken<'a> {
    pub(super) token: &'a UpstreamImportToken,
    pub(super) local_key_id: Option<String>,
    pub(super) name: String,
    pub(super) endpoint_formats: Vec<String>,
    pub(super) effective_cost_multiplier: Decimal,
}

#[cfg(test)]
impl<'a> SelectedToken<'a> {
    pub(super) fn for_test(token: &'a UpstreamImportToken, endpoint_formats: Vec<String>) -> Self {
        Self {
            token,
            local_key_id: None,
            name: token.name.clone(),
            endpoint_formats,
            effective_cost_multiplier: Decimal::ONE,
        }
    }

    pub(super) fn for_test_with_multiplier(token: &'a UpstreamImportToken, endpoint_formats: Vec<String>, effective_cost_multiplier: Decimal) -> Self {
        Self {
            token,
            local_key_id: None,
            name: token.name.clone(),
            endpoint_formats,
            effective_cost_multiplier,
        }
    }

    pub(super) fn for_test_with_local_key(token: &'a UpstreamImportToken, local_key_id: &str, endpoint_formats: Vec<String>) -> Self {
        Self {
            token,
            local_key_id: Some(local_key_id.to_owned()),
            name: token.name.clone(),
            endpoint_formats,
            effective_cost_multiplier: Decimal::ONE,
        }
    }
}

pub(super) fn selected_tokens<'a>(data: &'a UpstreamImportData, inputs: &[ProviderQuickImportSelectedToken]) -> ProviderResult<Vec<SelectedToken<'a>>> {
    if inputs.is_empty() {
        return Err(ProviderError::InvalidInput("selected_tokens cannot be empty".into()));
    }
    let by_id = data.tokens.iter().map(|token| (token.id.as_str(), token)).collect::<BTreeMap<_, _>>();
    inputs.iter().map(|input| selected_token(&by_id, input)).collect()
}

pub(super) fn selected_bind_tokens<'a>(
    data: &'a UpstreamImportData,
    inputs: &[ProviderQuickImportBindSelectedToken],
) -> ProviderResult<Vec<SelectedToken<'a>>> {
    if inputs.is_empty() {
        return Err(ProviderError::InvalidInput("selected_tokens cannot be empty".into()));
    }
    let by_id = data.tokens.iter().map(|token| (token.id.as_str(), token)).collect::<BTreeMap<_, _>>();
    inputs.iter().map(|input| selected_bind_token(&by_id, input)).collect()
}

pub(super) fn resolved_mappings(
    selected: &[SelectedToken<'_>],
    globals: &[GlobalModelResponse],
    selected_model_ids: Vec<String>,
    inputs: Vec<ProviderQuickImportModelMappingInput>,
) -> ProviderResult<BTreeMap<String, String>> {
    let by_name = globals_by_name(globals);
    let by_id = globals_by_id(globals);
    let submitted = submitted_mappings(inputs);
    let available = upstream_model_ids_from_selected(selected);
    let selected_ids = normalized_selected_model_ids(selected_model_ids);
    if selected_ids.is_empty() {
        return Err(ProviderError::InvalidInput("selected_model_ids cannot be empty".into()));
    }
    validate_selected_model_ids(&selected_ids, &available)?;
    validate_submitted_mappings_selected(&submitted, &selected_ids)?;

    let mut output = BTreeMap::new();
    for upstream_id in selected_ids {
        let global_id = submitted
            .get(&upstream_id)
            .cloned()
            .or_else(|| by_name.get(&upstream_id).map(|model| model.id.clone()));
        let Some(global_id) = global_id else {
            return Err(ProviderError::InvalidInput(format!("model mapping is required: {upstream_id}")));
        };
        global_model(&by_id, &global_id)?;
        output.insert(upstream_id, global_id);
    }
    Ok(output)
}

pub(super) fn allowed_model_ids(token: &SelectedToken<'_>, mappings: &BTreeMap<String, String>) -> ProviderResult<Vec<String>> {
    let mut ids = BTreeSet::new();
    for model in &token.token.models {
        if let Some(global_id) = mappings.get(&model.id) {
            ids.insert(global_id.to_owned());
        }
    }
    if ids.is_empty() {
        return Err(ProviderError::InvalidInput(format!("selected token has no mapped models: {}", token.token.id)));
    }
    Ok(ids.into_iter().collect())
}

pub(super) fn key_model_mappings(token: &SelectedToken<'_>, mappings: &BTreeMap<String, String>) -> Vec<ProviderQuickImportKeyModelCreate> {
    token
        .token
        .models
        .iter()
        .filter_map(|model| {
            mappings.get(&model.id).map(|global_model_id| ProviderQuickImportKeyModelCreate {
                upstream_model_id: model.id.clone(),
                global_model_id: global_model_id.clone(),
            })
        })
        .collect()
}

pub(super) fn assert_no_mapping_conflicts(mappings: &BTreeMap<String, String>) -> ProviderResult<()> {
    let mut seen = BTreeMap::<&str, &str>::new();
    for (upstream_id, global_id) in mappings {
        if let Some(previous) = seen.insert(global_id, upstream_id)
            && previous != upstream_id
        {
            return Err(ProviderError::InvalidInput(format!(
                "multiple upstream models map to the same global model: {global_id}"
            )));
        }
    }
    Ok(())
}

fn selected_token<'a>(by_id: &BTreeMap<&str, &'a UpstreamImportToken>, input: &ProviderQuickImportSelectedToken) -> ProviderResult<SelectedToken<'a>> {
    let token = by_id
        .get(input.upstream_token_id.trim())
        .copied()
        .ok_or_else(|| ProviderError::InvalidInput(format!("upstream token does not exist: {}", input.upstream_token_id)))?;
    validate_selected_token_fields(token, input.name.trim(), input.effective_cost_multiplier)?;
    Ok(SelectedToken {
        token,
        local_key_id: None,
        name: input.name.trim().to_owned(),
        endpoint_formats: normalized_formats(&input.endpoint_formats)?,
        effective_cost_multiplier: input.effective_cost_multiplier,
    })
}

fn selected_bind_token<'a>(by_id: &BTreeMap<&str, &'a UpstreamImportToken>, input: &ProviderQuickImportBindSelectedToken) -> ProviderResult<SelectedToken<'a>> {
    let token = by_id
        .get(input.upstream_token_id.trim())
        .copied()
        .ok_or_else(|| ProviderError::InvalidInput(format!("upstream token does not exist: {}", input.upstream_token_id)))?;
    validate_selected_token_fields(token, input.name.trim(), input.effective_cost_multiplier)?;
    Ok(SelectedToken {
        token,
        local_key_id: input.local_key_id.as_ref().map(|id| id.trim().to_owned()).filter(|id| !id.is_empty()),
        name: input.name.trim().to_owned(),
        endpoint_formats: normalized_formats(&input.endpoint_formats)?,
        effective_cost_multiplier: input.effective_cost_multiplier,
    })
}

fn validate_selected_token_fields(token: &UpstreamImportToken, name: &str, effective_cost_multiplier: Decimal) -> ProviderResult<()> {
    if token.status != 1 {
        return Err(ProviderError::InvalidInput(format!("upstream token is disabled: {}", token.id)));
    }
    if token.group.is_none() {
        return Err(ProviderError::InvalidInput(format!("upstream token group is missing: {}", token.id)));
    }
    if token.models.is_empty() {
        return Err(ProviderError::InvalidInput(format!("upstream token has no models: {}", token.id)));
    }
    if name.is_empty() {
        return Err(ProviderError::InvalidInput(format!("selected token name cannot be blank: {}", token.id)));
    }
    if effective_cost_multiplier <= Decimal::ZERO {
        return Err(ProviderError::InvalidInput(format!(
            "effective_cost_multiplier must be greater than 0: {}",
            token.id
        )));
    }
    Ok(())
}

fn normalized_formats(values: &[String]) -> ProviderResult<Vec<String>> {
    let formats = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if formats.is_empty() {
        return Err(ProviderError::InvalidInput("endpoint_formats cannot be empty".into()));
    }
    Ok(formats.into_iter().collect())
}

fn upstream_model_ids_from_selected(tokens: &[SelectedToken<'_>]) -> BTreeSet<String> {
    tokens
        .iter()
        .flat_map(|token| token.token.models.iter().map(|model| model.id.clone()))
        .collect()
}

fn normalized_selected_model_ids(values: Vec<String>) -> BTreeSet<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn validate_selected_model_ids(selected_ids: &BTreeSet<String>, available: &BTreeSet<String>) -> ProviderResult<()> {
    for upstream_id in selected_ids {
        if !available.contains(upstream_id) {
            return Err(ProviderError::InvalidInput(format!(
                "selected model does not exist on selected tokens: {upstream_id}"
            )));
        }
    }
    Ok(())
}

fn validate_submitted_mappings_selected(submitted: &BTreeMap<String, String>, selected_ids: &BTreeSet<String>) -> ProviderResult<()> {
    for upstream_id in submitted.keys() {
        if !selected_ids.contains(upstream_id) {
            return Err(ProviderError::InvalidInput(format!("model mapping is not selected for import: {upstream_id}")));
        }
    }
    Ok(())
}

fn submitted_mappings(inputs: Vec<ProviderQuickImportModelMappingInput>) -> BTreeMap<String, String> {
    inputs
        .into_iter()
        .map(|input| (input.upstream_model_id.trim().to_owned(), input.global_model_id.trim().to_owned()))
        .filter(|(upstream_id, global_id)| !upstream_id.is_empty() && !global_id.is_empty())
        .collect()
}
