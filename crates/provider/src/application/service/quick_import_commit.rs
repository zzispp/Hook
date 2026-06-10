use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderApiKeyCreate, ProviderCreate, ProviderEndpointCreate, ProviderModelBindingCreate, ProviderModelCostBatchUpsert, ProviderModelMapping,
        ProviderQuickImportModelMappingInput, ProviderQuickImportSelectedToken, ProviderQuickImportSourceConfig,
    },
};

use crate::application::{
    ProviderError, ProviderQuickImportApiKeyCreate, ProviderQuickImportCreate, ProviderQuickImportModelCostCreate, ProviderResult, SecretCipher,
    UpstreamImportData, UpstreamImportToken,
};

use super::{
    quick_import_costs::model_cost,
    quick_import_shared::{global_model, globals_by_id, globals_by_name, source_base_url},
};
use crate::application::validation::{sanitize_api_key, sanitize_endpoint, validate_api_key, validate_endpoint, validate_model_cost_batch};

pub(super) fn quick_import_create<C>(
    provider: ProviderCreate,
    source: &ProviderQuickImportSourceConfig,
    selected: Vec<SelectedToken<'_>>,
    globals: &[GlobalModelResponse],
    mappings: BTreeMap<String, String>,
    cipher: &C,
) -> ProviderResult<ProviderQuickImportCreate>
where
    C: SecretCipher,
{
    let global_by_id = globals_by_id(globals);
    Ok(ProviderQuickImportCreate {
        provider,
        endpoints: endpoint_creates(source_base_url(source), &selected)?,
        model_bindings: binding_creates(&mappings, &global_by_id)?,
        api_keys: key_creates(&selected, &mappings, cipher)?,
        model_costs: cost_creates(&selected, &mappings, &global_by_id)?,
    })
}

pub(super) fn selected_tokens<'a>(data: &'a UpstreamImportData, inputs: &[ProviderQuickImportSelectedToken]) -> ProviderResult<Vec<SelectedToken<'a>>> {
    if inputs.is_empty() {
        return Err(ProviderError::InvalidInput("selected_tokens cannot be empty".into()));
    }
    let by_id = data.tokens.iter().map(|token| (token.id.as_str(), token)).collect::<BTreeMap<_, _>>();
    inputs.iter().map(|input| selected_token(&by_id, input)).collect()
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

pub(super) struct SelectedToken<'a> {
    token: &'a UpstreamImportToken,
    name: String,
    endpoint_formats: Vec<String>,
    effective_cost_multiplier: Decimal,
}

#[cfg(test)]
impl<'a> SelectedToken<'a> {
    pub(super) fn for_test(token: &'a UpstreamImportToken, endpoint_formats: Vec<String>) -> Self {
        Self {
            token,
            name: token.name.clone(),
            endpoint_formats,
            effective_cost_multiplier: Decimal::ONE,
        }
    }

    pub(super) fn for_test_with_multiplier(token: &'a UpstreamImportToken, endpoint_formats: Vec<String>, effective_cost_multiplier: Decimal) -> Self {
        Self {
            token,
            name: token.name.clone(),
            endpoint_formats,
            effective_cost_multiplier,
        }
    }
}

fn endpoint_creates(base_url: String, selected: &[SelectedToken<'_>]) -> ProviderResult<Vec<ProviderEndpointCreate>> {
    let formats = selected.iter().flat_map(|token| token.endpoint_formats.iter()).collect::<BTreeSet<_>>();
    formats.into_iter().map(|format| endpoint_create(format, &base_url)).collect()
}

fn endpoint_create(format: &str, base_url: &str) -> ProviderResult<ProviderEndpointCreate> {
    let endpoint = sanitize_endpoint(ProviderEndpointCreate {
        api_format: format.to_owned(),
        base_url: base_url.to_owned(),
        custom_path: None,
        max_retries: None,
        is_active: Some(true),
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
    });
    validate_endpoint(&endpoint)?;
    Ok(endpoint)
}

fn binding_creates(
    mappings: &BTreeMap<String, String>,
    global_by_id: &BTreeMap<String, &GlobalModelResponse>,
) -> ProviderResult<Vec<ProviderModelBindingCreate>> {
    assert_no_mapping_conflicts(mappings)?;
    mappings
        .iter()
        .map(|(upstream_id, global_id)| binding_create(upstream_id, global_model(global_by_id, global_id)?))
        .collect()
}

fn key_creates<C>(selected: &[SelectedToken<'_>], mappings: &BTreeMap<String, String>, cipher: &C) -> ProviderResult<Vec<ProviderQuickImportApiKeyCreate>>
where
    C: SecretCipher,
{
    selected
        .iter()
        .map(|token| {
            let api_key = token.token.api_key.as_deref().ok_or_else(|| missing_key_error(&token.token.id))?;
            let input = sanitize_api_key(api_key_create(token, mappings, api_key)?);
            validate_api_key(&input)?;
            Ok(ProviderQuickImportApiKeyCreate {
                upstream_token_id: token.token.id.clone(),
                encrypted_api_key: cipher.encrypt_provider_key(api_key)?,
                input,
            })
        })
        .collect()
}

fn cost_creates(
    selected: &[SelectedToken<'_>],
    mappings: &BTreeMap<String, String>,
    global_by_id: &BTreeMap<String, &GlobalModelResponse>,
) -> ProviderResult<Vec<ProviderQuickImportModelCostCreate>> {
    let mut costs = Vec::new();
    for token in selected {
        push_token_costs(&mut costs, token, mappings, global_by_id)?;
    }
    validate_model_cost_batch(&ProviderModelCostBatchUpsert {
        costs: costs.iter().map(|item| item.cost.clone()).collect(),
    })?;
    Ok(costs)
}

fn push_token_costs(
    costs: &mut Vec<ProviderQuickImportModelCostCreate>,
    token: &SelectedToken<'_>,
    mappings: &BTreeMap<String, String>,
    global_by_id: &BTreeMap<String, &GlobalModelResponse>,
) -> ProviderResult<()> {
    for model in &token.token.models {
        let Some(global_id) = mappings.get(&model.id) else {
            continue;
        };
        let global = global_model(global_by_id, global_id)?;
        costs.push(ProviderQuickImportModelCostCreate {
            upstream_token_id: token.token.id.clone(),
            global_model_id: global.id.clone(),
            cost: model_cost(global, token.effective_cost_multiplier)?,
        });
    }
    Ok(())
}

fn selected_token<'a>(by_id: &BTreeMap<&str, &'a UpstreamImportToken>, input: &ProviderQuickImportSelectedToken) -> ProviderResult<SelectedToken<'a>> {
    let token = by_id
        .get(input.upstream_token_id.trim())
        .copied()
        .ok_or_else(|| ProviderError::InvalidInput(format!("upstream token does not exist: {}", input.upstream_token_id)))?;
    if token.status != 1 {
        return Err(ProviderError::InvalidInput(format!("upstream token is disabled: {}", token.id)));
    }
    if token.models.is_empty() {
        return Err(ProviderError::InvalidInput(format!("upstream token has no models: {}", token.id)));
    }
    if input.name.trim().is_empty() {
        return Err(ProviderError::InvalidInput(format!("selected token name cannot be blank: {}", token.id)));
    }
    if input.effective_cost_multiplier <= Decimal::ZERO {
        return Err(ProviderError::InvalidInput(format!(
            "effective_cost_multiplier must be greater than 0: {}",
            token.id
        )));
    }
    Ok(SelectedToken {
        token,
        name: input.name.trim().to_owned(),
        endpoint_formats: normalized_formats(&input.endpoint_formats)?,
        effective_cost_multiplier: input.effective_cost_multiplier,
    })
}

fn binding_create(upstream_id: &str, global: &GlobalModelResponse) -> ProviderResult<ProviderModelBindingCreate> {
    Ok(ProviderModelBindingCreate {
        global_model_id: global.id.clone(),
        provider_model_name: global.name.clone(),
        provider_model_mapping: (upstream_id != global.name).then(|| ProviderModelMapping {
            name: upstream_id.to_owned(),
            reasoning_effort: None,
        }),
        config: None,
    })
}

fn api_key_create(token: &SelectedToken<'_>, mappings: &BTreeMap<String, String>, api_key: &str) -> ProviderResult<ProviderApiKeyCreate> {
    Ok(ProviderApiKeyCreate {
        name: token.name.clone(),
        api_key: api_key.to_owned(),
        api_formats: token.endpoint_formats.clone(),
        allowed_model_ids: allowed_model_ids(token, mappings)?,
        note: token.token.group.as_ref().map(|group| format!("Imported from newapi group: {group}")),
        internal_priority: Some(10),
        rpm_limit: None,
        cache_ttl_minutes: Some(5),
        max_probe_interval_minutes: Some(32),
        time_range_enabled: Some(false),
        time_range_start: None,
        time_range_end: None,
        is_active: Some(true),
    })
}

fn allowed_model_ids(token: &SelectedToken<'_>, mappings: &BTreeMap<String, String>) -> ProviderResult<Vec<String>> {
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

fn missing_key_error(token_id: &str) -> ProviderError {
    ProviderError::Infrastructure(format!("newapi key was not fetched for selected token: {token_id}"))
}
