use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;
use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate, ProviderModelBindingCreate, ProviderModelCostBatchUpsert,
        ProviderQuickImportProviderConfig, ProviderQuickImportSourceConfig, ProviderQuickImportSyncConfig,
    },
};

use crate::application::{
    ProviderError, ProviderQuickImportApiKeyCreate, ProviderQuickImportAppend, ProviderQuickImportBind, ProviderQuickImportBoundApiKey,
    ProviderQuickImportCreate, ProviderQuickImportKeyReplacement, ProviderQuickImportModelCostCreate, ProviderQuickImportSyncSourceCreate, ProviderResult,
    SecretCipher, UpstreamImportToken,
};

use super::quick_import_commit_endpoints::endpoint_creates;
pub(super) use super::quick_import_commit_models::{SelectedToken, assert_no_mapping_conflicts, resolved_mappings, selected_bind_tokens, selected_tokens};
use super::quick_import_commit_models::{allowed_model_ids, key_model_mappings};
use super::{
    quick_import_costs::model_cost,
    quick_import_shared::{global_model, globals_by_id, source_base_url},
};
use crate::application::validation::{sanitize_api_key, validate_api_key, validate_model_cost_batch};

pub(super) struct QuickImportCreateDraft<'a, C> {
    pub(super) provider: ProviderCreate,
    pub(super) provider_config: &'a ProviderQuickImportProviderConfig,
    pub(super) source: &'a ProviderQuickImportSourceConfig,
    pub(super) recharge_multiplier: Decimal,
    pub(super) sync_config: ProviderQuickImportSyncConfig,
    pub(super) selected: Vec<SelectedToken<'a>>,
    pub(super) globals: &'a [GlobalModelResponse],
    pub(super) mappings: BTreeMap<String, String>,
    pub(super) cipher: &'a C,
}

pub(super) struct QuickImportAppendDraft<'a, C> {
    pub(super) provider_id: String,
    pub(super) source_id: String,
    pub(super) source: &'a ProviderQuickImportSourceConfig,
    pub(super) selected: Vec<SelectedToken<'a>>,
    pub(super) globals: &'a [GlobalModelResponse],
    pub(super) mappings: BTreeMap<String, String>,
    pub(super) cipher: &'a C,
}

pub(super) struct QuickImportBindDraft<'a, C> {
    pub(super) provider_id: String,
    pub(super) source: &'a ProviderQuickImportSourceConfig,
    pub(super) provider_config: &'a ProviderQuickImportProviderConfig,
    pub(super) recharge_multiplier: Decimal,
    pub(super) sync_config: ProviderQuickImportSyncConfig,
    pub(super) selected: Vec<SelectedToken<'a>>,
    pub(super) globals: &'a [GlobalModelResponse],
    pub(super) mappings: BTreeMap<String, String>,
    pub(super) cipher: &'a C,
}

pub(super) struct QuickImportKeyReplacementDraft<'a> {
    pub(super) provider_id: String,
    pub(super) source_id: String,
    pub(super) key_id: String,
    pub(super) token: &'a UpstreamImportToken,
    pub(super) effective_cost_multiplier: Decimal,
    pub(super) globals: &'a [GlobalModelResponse],
    pub(super) mappings: BTreeMap<String, String>,
    pub(super) encrypted_api_key: Option<String>,
    pub(super) existing_global_model_ids: &'a BTreeSet<String>,
}

pub(super) fn quick_import_create<C>(input: QuickImportCreateDraft<'_, C>) -> ProviderResult<ProviderQuickImportCreate>
where
    C: SecretCipher,
{
    let global_by_id = globals_by_id(input.globals);
    Ok(ProviderQuickImportCreate {
        provider: input.provider,
        sync_source: Some(sync_source_create(input.source, input.recharge_multiplier, input.sync_config, input.cipher)?),
        endpoints: endpoint_creates(
            source_base_url(input.source),
            &input.selected,
            input.provider_config.upstream_image_native_stream.unwrap_or(false),
        )?,
        model_bindings: binding_creates(&input.mappings, &global_by_id)?,
        api_keys: key_creates(&input.selected, &input.mappings, input.cipher)?,
        model_costs: cost_creates(&input.selected, &input.mappings, &global_by_id)?,
    })
}

pub(super) fn quick_import_append<C>(input: QuickImportAppendDraft<'_, C>) -> ProviderResult<ProviderQuickImportAppend>
where
    C: SecretCipher,
{
    let global_by_id = globals_by_id(input.globals);
    Ok(ProviderQuickImportAppend {
        provider_id: input.provider_id,
        source_id: input.source_id,
        endpoints: endpoint_creates(source_base_url(input.source), &input.selected, false)?,
        model_bindings: binding_creates(&input.mappings, &global_by_id)?,
        api_keys: key_creates(&input.selected, &input.mappings, input.cipher)?,
        model_costs: cost_creates(&input.selected, &input.mappings, &global_by_id)?,
    })
}

pub(super) fn quick_import_bind<C>(input: QuickImportBindDraft<'_, C>) -> ProviderResult<ProviderQuickImportBind>
where
    C: SecretCipher,
{
    let global_by_id = globals_by_id(input.globals);
    Ok(ProviderQuickImportBind {
        provider_id: input.provider_id,
        sync_source: sync_source_create(input.source, input.recharge_multiplier, input.sync_config, input.cipher)?,
        endpoints: endpoint_creates(
            source_base_url(input.source),
            &input.selected,
            input.provider_config.upstream_image_native_stream.unwrap_or(false),
        )?,
        model_bindings: binding_creates(&input.mappings, &global_by_id)?,
        api_keys: bound_key_creates(&input.selected, &input.mappings, input.cipher)?,
        model_costs: cost_creates(&input.selected, &input.mappings, &global_by_id)?,
    })
}

pub(super) fn quick_import_key_replacement(input: QuickImportKeyReplacementDraft<'_>) -> ProviderResult<ProviderQuickImportKeyReplacement> {
    let global_by_id = globals_by_id(input.globals);
    let selected = SelectedToken {
        token: input.token,
        local_key_id: Some(input.key_id.clone()),
        name: input.token.name.clone(),
        endpoint_formats: Vec::new(),
        effective_cost_multiplier: input.effective_cost_multiplier,
    };
    let model_bindings = binding_creates(&input.mappings, &global_by_id)?
        .into_iter()
        .filter(|binding| !input.existing_global_model_ids.contains(&binding.global_model_id))
        .collect();
    Ok(ProviderQuickImportKeyReplacement {
        provider_id: input.provider_id,
        source_id: input.source_id,
        key_id: input.key_id,
        upstream_token_id: input.token.id.clone(),
        upstream_token_name: input.token.name.clone(),
        upstream_masked_key: input.token.masked_key.clone(),
        upstream_group: input.token.group.clone(),
        upstream_group_ratio: input.token.group_ratio,
        effective_cost_multiplier: input.effective_cost_multiplier,
        model_mappings: key_model_mappings(&selected, &input.mappings),
        input: ProviderApiKeyUpdate {
            allowed_model_ids: Some(allowed_model_ids(&selected, &input.mappings)?),
            is_active: Some(true),
            ..ProviderApiKeyUpdate::default()
        },
        encrypted_api_key: input.encrypted_api_key,
        model_bindings,
        model_costs: cost_creates(std::slice::from_ref(&selected), &input.mappings, &global_by_id)?,
    })
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
    selected.iter().map(|token| key_create(token, mappings, cipher)).collect()
}

fn bound_key_creates<C>(selected: &[SelectedToken<'_>], mappings: &BTreeMap<String, String>, cipher: &C) -> ProviderResult<Vec<ProviderQuickImportBoundApiKey>>
where
    C: SecretCipher,
{
    selected
        .iter()
        .map(|token| {
            let create = key_create(token, mappings, cipher)?;
            Ok(ProviderQuickImportBoundApiKey {
                local_key_id: token.local_key_id.clone(),
                create,
            })
        })
        .collect()
}

fn key_create<C>(token: &SelectedToken<'_>, mappings: &BTreeMap<String, String>, cipher: &C) -> ProviderResult<ProviderQuickImportApiKeyCreate>
where
    C: SecretCipher,
{
    let api_key = token.token.api_key.as_deref().ok_or_else(|| missing_key_error(&token.token.id))?;
    let input = sanitize_api_key(api_key_create(token, mappings, api_key)?);
    validate_api_key(&input)?;
    Ok(ProviderQuickImportApiKeyCreate {
        upstream_token_id: token.token.id.clone(),
        upstream_token_name: token.token.name.clone(),
        upstream_masked_key: token.token.masked_key.clone(),
        upstream_group: token.token.group.clone(),
        upstream_group_ratio: token.token.group_ratio,
        effective_cost_multiplier: token.effective_cost_multiplier,
        model_mappings: key_model_mappings(token, mappings),
        encrypted_api_key: cipher.encrypt_provider_key(api_key)?,
        input,
    })
}

fn sync_source_create<C>(
    source: &ProviderQuickImportSourceConfig,
    recharge_multiplier: Decimal,
    sync_config: ProviderQuickImportSyncConfig,
    cipher: &C,
) -> ProviderResult<ProviderQuickImportSyncSourceCreate>
where
    C: SecretCipher,
{
    let ProviderQuickImportSourceConfig::Newapi(config) = source;
    Ok(ProviderQuickImportSyncSourceCreate {
        source_kind: source.kind(),
        base_url: source_base_url(source),
        encrypted_system_access_token: cipher.encrypt_provider_key(config.system_access_token.trim())?,
        user_id: config.user_id.trim().to_owned(),
        recharge_multiplier,
        sync_config,
    })
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

fn binding_create(_upstream_id: &str, global: &GlobalModelResponse) -> ProviderResult<ProviderModelBindingCreate> {
    Ok(ProviderModelBindingCreate {
        global_model_id: global.id.clone(),
        is_active: Some(true),
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

fn missing_key_error(token_id: &str) -> ProviderError {
    ProviderError::Infrastructure(format!("newapi key was not fetched for selected token: {token_id}"))
}
