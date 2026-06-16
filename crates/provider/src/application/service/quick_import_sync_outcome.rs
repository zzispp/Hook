use std::collections::BTreeMap;

use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderModelCostUpsert, ProviderQuickImportCostSyncMode, ProviderQuickImportGroupChangedAction, ProviderQuickImportSourceConfig,
        ProviderQuickImportSyncStatus, ProviderQuickImportUpstreamAnomalyAction,
    },
};

use crate::application::{
    ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyPatch, ProviderQuickImportSyncSource, UpstreamImportModel,
    UpstreamProviderImportSource, UpstreamSyncSnapshot, UpstreamSyncToken,
};

use super::{
    quick_import_sync_bindings::BindingInfo,
    quick_import_sync_candidates::candidate_model_ids,
    quick_import_sync_group_ratio::{GroupRatioLookup, group_ratio},
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

impl KeyOutcome {
    pub(super) fn patch(&self, key_id: String) -> ProviderQuickImportSyncKeyPatch {
        ProviderQuickImportSyncKeyPatch {
            key_id,
            statuses: self.statuses.clone(),
            upstream_group: self.upstream_group.clone(),
            upstream_group_ratio: self.group_ratio_patch,
            effective_cost_multiplier: self.effective_multiplier_patch,
            last_error: self.error.clone(),
        }
    }

    pub(super) fn synced_group(&self) -> Option<&Option<String>> {
        self.upstream_group.as_ref()
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
        );
    };
    if token.status != 1 {
        return anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamTokenDisabled,
            source.sync_config.anomaly_actions.token_disabled,
        );
    }
    if token.group.as_deref() != key.upstream_group.as_deref() {
        return group_changed_outcome(&context, key, token).await;
    }
    let ratio = match group_ratio(snapshot, key.upstream_group.as_deref()) {
        GroupRatioLookup::Fixed(value) => value,
        GroupRatioLookup::Missing => {
            return anomaly_outcome(
                ProviderQuickImportSyncStatus::UpstreamGroupRemoved,
                source.sync_config.anomaly_actions.group_removed,
            );
        }
        GroupRatioLookup::NonFixed(value) => return non_fixed_ratio_outcome(key.upstream_group.as_deref(), value),
    };
    match check_models(importer, source_config, key).await {
        Ok(ModelCheck::Available(models)) => cost_outcome(source, globals, bindings, key, None, ratio, &models),
        Ok(ModelCheck::Removed) => anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamModelRemoved,
            source.sync_config.anomaly_actions.model_removed,
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
    let ratio = match group_ratio(context.snapshot, token.group.as_deref()) {
        GroupRatioLookup::Fixed(value) => value,
        GroupRatioLookup::Missing => {
            return anomaly_outcome(
                ProviderQuickImportSyncStatus::UpstreamGroupRemoved,
                context.source.sync_config.anomaly_actions.group_removed,
            );
        }
        GroupRatioLookup::NonFixed(value) => return non_fixed_ratio_outcome(token.group.as_deref(), value),
    };
    match check_models(context.importer, context.source_config, key).await {
        Ok(ModelCheck::Available(models)) => cost_outcome(
            context.source,
            context.globals,
            context.bindings,
            key,
            Some(token.group.clone()),
            ratio,
            &models,
        ),
        Ok(ModelCheck::Removed) => anomaly_outcome(
            ProviderQuickImportSyncStatus::UpstreamModelRemoved,
            context.source.sync_config.anomaly_actions.model_removed,
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

fn cost_outcome(
    source: &ProviderQuickImportSyncSource,
    globals: &BTreeMap<String, GlobalModelResponse>,
    bindings: &BTreeMap<String, BindingInfo>,
    key: &ProviderQuickImportSyncKey,
    upstream_group: Option<Option<String>>,
    group_ratio: rust_decimal::Decimal,
    upstream_models: &[UpstreamImportModel],
) -> KeyOutcome {
    let group_ratio = persisted_multiplier(group_ratio);
    let effective = persisted_multiplier(group_ratio / source.recharge_multiplier);
    let costs = match costs_for_key(globals, bindings, key, effective) {
        Ok(costs) => costs,
        Err(error) => return cost_error(group_ratio, effective, upstream_group, error),
    };
    let candidates = candidate_model_ids(globals, bindings, key, upstream_models);
    if source.sync_config.cost_sync_mode == ProviderQuickImportCostSyncMode::ReportOnly {
        return report_only_outcome(group_ratio, effective, upstream_group, key.effective_cost_multiplier, candidates);
    }
    let statuses = statuses_with_candidates(vec![ProviderQuickImportSyncStatus::Ok], &candidates);
    KeyOutcome {
        statuses,
        costs: Some(costs),
        disable_key: false,
        observed_group: upstream_group.clone(),
        observed_group_ratio: Some(group_ratio),
        observed_effective_multiplier: Some(effective),
        candidate_model_ids: candidates,
        upstream_group,
        group_ratio_patch: Some(group_ratio),
        effective_multiplier_patch: Some(effective),
        error: None,
    }
}

fn report_only_outcome(
    group_ratio: rust_decimal::Decimal,
    effective: rust_decimal::Decimal,
    upstream_group: Option<Option<String>>,
    current: rust_decimal::Decimal,
    candidates: Vec<String>,
) -> KeyOutcome {
    let base_statuses = if effective == current {
        vec![ProviderQuickImportSyncStatus::Ok]
    } else {
        vec![ProviderQuickImportSyncStatus::CostPendingUpdate]
    };
    let statuses = statuses_with_candidates(base_statuses, &candidates);
    KeyOutcome {
        statuses,
        costs: None,
        disable_key: false,
        observed_group: upstream_group.clone(),
        observed_group_ratio: Some(group_ratio),
        observed_effective_multiplier: Some(effective),
        candidate_model_ids: candidates,
        upstream_group,
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

fn anomaly_outcome(status: ProviderQuickImportSyncStatus, action: ProviderQuickImportUpstreamAnomalyAction) -> KeyOutcome {
    let mut outcome = status_base(status);
    outcome.disable_key = action == ProviderQuickImportUpstreamAnomalyAction::DisableKey;
    outcome
}

fn anomaly_error(status: ProviderQuickImportSyncStatus, action: ProviderQuickImportUpstreamAnomalyAction, error: ProviderError) -> KeyOutcome {
    let mut outcome = anomaly_outcome(status, action);
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
        upstream_group: None,
        group_ratio_patch: None,
        effective_multiplier_patch: None,
        error: None,
    }
}

fn token_by_id<'a>(snapshot: &'a UpstreamSyncSnapshot, id: &str) -> Option<&'a UpstreamSyncToken> {
    snapshot.tokens.iter().find(|token| token.id == id)
}
