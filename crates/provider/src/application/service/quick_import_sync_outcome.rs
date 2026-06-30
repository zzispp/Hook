use std::collections::BTreeMap;

use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderModelCostUpsert, ProviderQuickImportCostSyncMode, ProviderQuickImportGroupChangedAction, ProviderQuickImportSourceConfig,
        ProviderQuickImportSyncStatus, ProviderQuickImportUpstreamAnomalyAction, ProviderQuickImportUpstreamModelSnapshot,
    },
};

use crate::application::{
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyPatch, ProviderQuickImportSyncSource, UpstreamImportModel,
    UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

use super::{
    quick_import_sync_bindings::BindingInfo,
    quick_import_sync_candidates::candidate_model_ids,
    quick_import_sync_group_ratio::{GroupRatioLookup, group_ratio, same_group},
    quick_import_sync_key_costs::costs_for_key,
    quick_import_sync_model_check::{ModelCheck, check_models},
    quick_import_sync_outcome_support::{persisted_multiplier, statuses_with_candidates},
};

pub(super) struct KeyOutcome {
    pub(super) statuses: Vec<ProviderQuickImportSyncStatus>,
    pub(super) costs: Option<Vec<ProviderModelCostUpsert>>,
    pub(super) disable_key: bool,
    pub(super) observed_group: Option<Option<String>>,
    pub(super) observed_group_ratio: Option<rust_decimal::Decimal>,
    pub(super) observed_effective_multiplier: Option<rust_decimal::Decimal>,
    pub(super) candidate_model_ids: Vec<String>,
    pub(super) missing_upstream_model_ids: Vec<String>,
    pub(super) upstream_models_snapshot: Vec<ProviderQuickImportUpstreamModelSnapshot>,
    group_change_synced: bool,
    upstream_group_id: Option<Option<String>>,
    upstream_group: Option<Option<String>>,
    group_ratio_patch: Option<rust_decimal::Decimal>,
    effective_multiplier_patch: Option<rust_decimal::Decimal>,
    error: Option<String>,
}

struct OutcomeContext<'a, I> {
    importer: &'a I,
    source: &'a ProviderQuickImportSyncSource,
    source_config: &'a ProviderQuickImportSourceConfig,
    snapshot: &'a UpstreamSyncSnapshot,
    globals: &'a BTreeMap<String, GlobalModelResponse>,
    bindings: &'a BTreeMap<String, BindingInfo>,
}

#[derive(Clone, Default)]
struct GroupPatch {
    upstream_group_id: Option<Option<String>>,
    upstream_group: Option<Option<String>>,
}

struct CostOutcomeInput<'a> {
    source: &'a ProviderQuickImportSyncSource,
    globals: &'a BTreeMap<String, GlobalModelResponse>,
    bindings: &'a BTreeMap<String, BindingInfo>,
    key: &'a ProviderQuickImportSyncKey,
    observed_group: Option<Option<String>>,
    group_patch: GroupPatch,
    group_ratio: rust_decimal::Decimal,
    upstream_models: &'a [UpstreamImportModel],
}

impl KeyOutcome {
    pub(super) fn patch(&self, key_id: String) -> ProviderQuickImportSyncKeyPatch {
        ProviderQuickImportSyncKeyPatch {
            key_id,
            statuses: self.statuses.clone(),
            upstream_group_id: self.upstream_group_id.clone(),
            upstream_group: self.upstream_group.clone(),
            upstream_group_ratio: self.group_ratio_patch,
            effective_cost_multiplier: self.effective_multiplier_patch,
            last_error: self.error.clone(),
        }
    }

    pub(super) fn synced_group(&self) -> Option<&Option<String>> {
        self.group_change_synced.then_some(self.upstream_group.as_ref()).flatten()
    }

    pub(super) fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

pub(super) async fn key_outcome<I>(
    importer: &I,
    source: &ProviderQuickImportSyncSource,
    source_config: &ProviderQuickImportSourceConfig,
    snapshot: &UpstreamSyncSnapshot,
    globals: &BTreeMap<String, GlobalModelResponse>,
    bindings: &BTreeMap<String, BindingInfo>,
    key: &ProviderQuickImportSyncKey,
) -> KeyOutcome
where
    I: UpstreamProviderImportSource,
{
    let context = OutcomeContext {
        importer,
        source,
        source_config,
        snapshot,
        globals,
        bindings,
    };
    if key.model_mappings.is_empty() {
        let mut outcome = status_base(ProviderQuickImportSyncStatus::NoAssociatedModels);
        outcome.disable_key = true;
        return outcome;
    }
    let Some(token) = token_by_id(snapshot, &key.upstream_token_id) else {
        return anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamTokenDeleted,
            source.sync_config.anomaly_actions.token_deleted,
            Vec::new(),
            Vec::new(),
        );
    };
    if !token.is_active {
        return anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamTokenDisabled,
            source.sync_config.anomaly_actions.token_disabled,
            Vec::new(),
            Vec::new(),
        );
    }
    if !same_group(
        key.upstream_group_id.as_deref(),
        key.upstream_group.as_deref(),
        token.group_id.as_deref(),
        token.group.as_deref(),
    ) {
        return group_changed_outcome(&context, key, token).await;
    }
    let ratio = match group_ratio(snapshot, token.group_id.as_deref(), token.group.as_deref()) {
        GroupRatioLookup::Fixed(value) => value,
        GroupRatioLookup::Missing => {
            return anomaly_outcome(
                ProviderQuickImportSyncStatus::UpstreamGroupRemoved,
                source.sync_config.anomaly_actions.group_removed,
                Vec::new(),
                Vec::new(),
            );
        }
        GroupRatioLookup::NonFixed(value) => return non_fixed_ratio_outcome(token.group.as_deref(), value),
    };
    let group_patch = group_patch(key, token);
    match check_models(importer, source_config, key).await {
        Ok(ModelCheck::Available { upstream_models }) => cost_outcome(CostOutcomeInput {
            source,
            globals,
            bindings,
            key,
            observed_group: None,
            group_patch,
            group_ratio: ratio,
            upstream_models: &upstream_models,
        }),
        Ok(ModelCheck::Removed {
            missing_upstream_model_ids,
            upstream_models,
        }) => anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamModelRemoved,
            source.sync_config.anomaly_actions.model_removed,
            missing_upstream_model_ids,
            models_snapshot(&upstream_models),
        ),
        Err(error) => anomaly_error(
            ProviderQuickImportSyncStatus::UpstreamKeyUnavailable,
            source.sync_config.anomaly_actions.key_unavailable,
            error,
        ),
    }
}

async fn group_changed_outcome<I>(context: &OutcomeContext<'_, I>, key: &ProviderQuickImportSyncKey, token: &UpstreamSyncToken) -> KeyOutcome
where
    I: UpstreamProviderImportSource,
{
    if context.source.sync_config.anomaly_actions.group_changed != ProviderQuickImportGroupChangedAction::Sync {
        return group_changed_status(context.source, token);
    }
    let ratio = match group_ratio(context.snapshot, token.group_id.as_deref(), token.group.as_deref()) {
        GroupRatioLookup::Fixed(value) => value,
        GroupRatioLookup::Missing => {
            return anomaly_outcome(
                ProviderQuickImportSyncStatus::UpstreamGroupRemoved,
                context.source.sync_config.anomaly_actions.group_removed,
                Vec::new(),
                Vec::new(),
            );
        }
        GroupRatioLookup::NonFixed(value) => return non_fixed_ratio_outcome(token.group.as_deref(), value),
    };
    let group_patch = group_patch(key, token);
    match check_models(context.importer, context.source_config, key).await {
        Ok(ModelCheck::Available { upstream_models }) => cost_outcome(CostOutcomeInput {
            source: context.source,
            globals: context.globals,
            bindings: context.bindings,
            key,
            observed_group: Some(token.group.clone()),
            group_patch,
            group_ratio: ratio,
            upstream_models: &upstream_models,
        }),
        Ok(ModelCheck::Removed {
            missing_upstream_model_ids,
            upstream_models,
        }) => anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamModelRemoved,
            context.source.sync_config.anomaly_actions.model_removed,
            missing_upstream_model_ids,
            models_snapshot(&upstream_models),
        ),
        Err(error) => anomaly_error(
            ProviderQuickImportSyncStatus::UpstreamKeyUnavailable,
            context.source.sync_config.anomaly_actions.key_unavailable,
            error,
        ),
    }
}

fn group_changed_status(source: &ProviderQuickImportSyncSource, token: &UpstreamSyncToken) -> KeyOutcome {
    let mut outcome = status_base(ProviderQuickImportSyncStatus::UpstreamGroupChanged);
    outcome.disable_key = source.sync_config.anomaly_actions.group_changed == ProviderQuickImportGroupChangedAction::DisableKey;
    outcome.observed_group = Some(token.group.clone());
    outcome
}

fn cost_outcome(input: CostOutcomeInput<'_>) -> KeyOutcome {
    let CostOutcomeInput {
        source,
        globals,
        bindings,
        key,
        observed_group,
        group_patch,
        group_ratio,
        upstream_models,
    } = input;
    let group_ratio = persisted_multiplier(group_ratio);
    let effective = persisted_multiplier(group_ratio / source.recharge_multiplier);
    let costs = match costs_for_key(globals, bindings, key, effective) {
        Ok(costs) => costs,
        Err(error) => return cost_error(group_ratio, effective, observed_group, error),
    };
    let candidates = candidate_model_ids(globals, key, upstream_models);
    let group_change_synced = observed_group.is_some();
    if source.sync_config.cost_sync_mode == ProviderQuickImportCostSyncMode::ReportOnly {
        return report_only_outcome(
            group_ratio,
            effective,
            observed_group,
            group_patch,
            key.effective_cost_multiplier,
            candidates,
            models_snapshot(upstream_models),
        );
    }
    let statuses = statuses_with_candidates(vec![ProviderQuickImportSyncStatus::Ok], &candidates);
    KeyOutcome {
        statuses,
        costs: Some(costs),
        disable_key: false,
        observed_group,
        observed_group_ratio: Some(group_ratio),
        observed_effective_multiplier: Some(effective),
        candidate_model_ids: candidates,
        missing_upstream_model_ids: Vec::new(),
        upstream_models_snapshot: models_snapshot(upstream_models),
        group_change_synced,
        upstream_group_id: group_patch.upstream_group_id,
        upstream_group: group_patch.upstream_group,
        group_ratio_patch: Some(group_ratio),
        effective_multiplier_patch: Some(effective),
        error: None,
    }
}

fn report_only_outcome(
    group_ratio: rust_decimal::Decimal,
    effective: rust_decimal::Decimal,
    observed_group: Option<Option<String>>,
    group_patch: GroupPatch,
    current: rust_decimal::Decimal,
    candidates: Vec<String>,
    upstream_models_snapshot: Vec<ProviderQuickImportUpstreamModelSnapshot>,
) -> KeyOutcome {
    let base_statuses = if effective == current {
        vec![ProviderQuickImportSyncStatus::Ok]
    } else {
        vec![ProviderQuickImportSyncStatus::CostPendingUpdate]
    };
    let statuses = statuses_with_candidates(base_statuses, &candidates);
    let group_change_synced = observed_group.is_some();
    KeyOutcome {
        statuses,
        costs: None,
        disable_key: false,
        observed_group,
        observed_group_ratio: Some(group_ratio),
        observed_effective_multiplier: Some(effective),
        candidate_model_ids: candidates,
        missing_upstream_model_ids: Vec::new(),
        upstream_models_snapshot,
        group_change_synced,
        upstream_group_id: group_patch.upstream_group_id,
        upstream_group: group_patch.upstream_group,
        group_ratio_patch: None,
        effective_multiplier_patch: None,
        error: None,
    }
}

fn cost_error(
    group_ratio: rust_decimal::Decimal,
    effective: rust_decimal::Decimal,
    upstream_group: Option<Option<String>>,
    error: ProviderError,
) -> KeyOutcome {
    let mut outcome = status_base(ProviderQuickImportSyncStatus::CostUnavailable);
    outcome.observed_group = upstream_group;
    outcome.observed_group_ratio = Some(group_ratio);
    outcome.observed_effective_multiplier = Some(effective);
    outcome.error = Some(error.to_string());
    outcome
}

fn non_fixed_ratio_outcome(group: Option<&str>, upstream_value: String) -> KeyOutcome {
    let mut outcome = status_base(ProviderQuickImportSyncStatus::CostUnavailable);
    let group = group.unwrap_or("未设置");
    outcome.error = Some(format!("newapi group ratio is not fixed for group {group}: {upstream_value}"));
    outcome
}

fn anomaly_outcome(
    status: ProviderQuickImportSyncStatus,
    action: ProviderQuickImportUpstreamAnomalyAction,
    missing_upstream_model_ids: Vec<String>,
    upstream_models_snapshot: Vec<ProviderQuickImportUpstreamModelSnapshot>,
) -> KeyOutcome {
    let mut outcome = status_base(status);
    outcome.disable_key = action == ProviderQuickImportUpstreamAnomalyAction::DisableKey;
    outcome.missing_upstream_model_ids = missing_upstream_model_ids;
    outcome.upstream_models_snapshot = upstream_models_snapshot;
    outcome
}

fn anomaly_error(status: ProviderQuickImportSyncStatus, action: ProviderQuickImportUpstreamAnomalyAction, error: ProviderError) -> KeyOutcome {
    let mut outcome = anomaly_outcome(status, action, Vec::new(), Vec::new());
    outcome.error = Some(error.to_string());
    outcome
}

fn status_base(status: ProviderQuickImportSyncStatus) -> KeyOutcome {
    KeyOutcome {
        statuses: vec![status],
        costs: None,
        disable_key: false,
        observed_group: None,
        observed_group_ratio: None,
        observed_effective_multiplier: None,
        candidate_model_ids: Vec::new(),
        missing_upstream_model_ids: Vec::new(),
        upstream_models_snapshot: Vec::new(),
        group_change_synced: false,
        upstream_group_id: None,
        upstream_group: None,
        group_ratio_patch: None,
        effective_multiplier_patch: None,
        error: None,
    }
}

fn models_snapshot(upstream_models: &[UpstreamImportModel]) -> Vec<ProviderQuickImportUpstreamModelSnapshot> {
    upstream_models
        .iter()
        .map(|model| ProviderQuickImportUpstreamModelSnapshot {
            upstream_model_id: model.id.clone(),
            supported_endpoint_types: model.supported_endpoint_types.clone(),
        })
        .collect()
}

fn token_by_id<'a>(snapshot: &'a UpstreamSyncSnapshot, id: &str) -> Option<&'a UpstreamSyncToken> {
    snapshot.tokens.iter().find(|token| token.id == id)
}

fn group_patch(key: &ProviderQuickImportSyncKey, token: &UpstreamSyncToken) -> GroupPatch {
    GroupPatch {
        upstream_group_id: (key.upstream_group_id != token.group_id).then(|| token.group_id.clone()),
        upstream_group: (key.upstream_group != token.group).then(|| token.group.clone()),
    }
}
