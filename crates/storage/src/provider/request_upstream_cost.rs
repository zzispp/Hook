use rust_decimal::Decimal;
use sea_orm::{ActiveValue, Set};
use types::model::PatchField;
use types::provider::{ProviderModelCostMode, ProviderModelCostSource, RequestUpstreamCost};

use crate::{StorageError, StorageResult};

use super::{
    record::{request_candidates, request_records},
    types::RequestUpstreamCostRecordPatch,
};

#[derive(Clone, Debug, Default)]
pub(super) struct StoredUpstreamCost {
    pub cost_mode: Option<String>,
    pub cost_source: Option<String>,
    pub price_per_request: Option<Decimal>,
    pub input_price_per_million: Option<Decimal>,
    pub output_price_per_million: Option<Decimal>,
    pub cache_creation_price_per_million: Option<Decimal>,
    pub cache_read_price_per_million: Option<Decimal>,
    pub request_cost: Option<Decimal>,
    pub input_cost: Option<Decimal>,
    pub output_cost: Option<Decimal>,
    pub cache_creation_cost: Option<Decimal>,
    pub cache_read_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
}

impl StoredUpstreamCost {
    pub(super) fn from_request_record(record: &request_records::Model) -> Self {
        Self {
            cost_mode: record.upstream_cost_mode.clone(),
            cost_source: record.upstream_cost_source.clone(),
            price_per_request: record.upstream_price_per_request,
            input_price_per_million: record.upstream_input_price_per_million,
            output_price_per_million: record.upstream_output_price_per_million,
            cache_creation_price_per_million: record.upstream_cache_creation_price_per_million,
            cache_read_price_per_million: record.upstream_cache_read_price_per_million,
            request_cost: record.upstream_request_cost,
            input_cost: record.upstream_input_cost,
            output_cost: record.upstream_output_cost,
            cache_creation_cost: record.upstream_cache_creation_cost,
            cache_read_cost: record.upstream_cache_read_cost,
            total_cost: record.upstream_total_cost,
        }
    }

    pub(super) fn from_candidate_record(record: &request_candidates::Model) -> Self {
        Self {
            cost_mode: record.upstream_cost_mode.clone(),
            cost_source: record.upstream_cost_source.clone(),
            price_per_request: record.upstream_price_per_request,
            input_price_per_million: record.upstream_input_price_per_million,
            output_price_per_million: record.upstream_output_price_per_million,
            cache_creation_price_per_million: record.upstream_cache_creation_price_per_million,
            cache_read_price_per_million: record.upstream_cache_read_price_per_million,
            request_cost: record.upstream_request_cost,
            input_cost: record.upstream_input_cost,
            output_cost: record.upstream_output_cost,
            cache_creation_cost: record.upstream_cache_creation_cost,
            cache_read_cost: record.upstream_cache_read_cost,
            total_cost: record.upstream_total_cost,
        }
    }
}

pub(super) fn response(input: StoredUpstreamCost) -> StorageResult<RequestUpstreamCost> {
    Ok(RequestUpstreamCost {
        upstream_cost_mode: parse_mode(input.cost_mode)?,
        upstream_cost_source: parse_source(input.cost_source)?,
        upstream_price_per_request: input.price_per_request,
        upstream_input_price_per_million: input.input_price_per_million,
        upstream_output_price_per_million: input.output_price_per_million,
        upstream_cache_creation_price_per_million: input.cache_creation_price_per_million,
        upstream_cache_read_price_per_million: input.cache_read_price_per_million,
        upstream_request_cost: input.request_cost,
        upstream_input_cost: input.input_cost,
        upstream_output_cost: input.output_cost,
        upstream_cache_creation_cost: input.cache_creation_cost,
        upstream_cache_read_cost: input.cache_read_cost,
        upstream_total_cost: input.total_cost,
    })
}

pub(super) fn apply_request_values(active: &mut request_records::ActiveModel, values: RequestUpstreamCost) {
    active.upstream_cost_mode = Set(values.upstream_cost_mode.map(|value| mode_value(&value).to_owned()));
    active.upstream_cost_source = Set(values.upstream_cost_source.map(|value| source_value(&value).to_owned()));
    active.upstream_price_per_request = Set(values.upstream_price_per_request);
    active.upstream_input_price_per_million = Set(values.upstream_input_price_per_million);
    active.upstream_output_price_per_million = Set(values.upstream_output_price_per_million);
    active.upstream_cache_creation_price_per_million = Set(values.upstream_cache_creation_price_per_million);
    active.upstream_cache_read_price_per_million = Set(values.upstream_cache_read_price_per_million);
    active.upstream_request_cost = Set(values.upstream_request_cost);
    active.upstream_input_cost = Set(values.upstream_input_cost);
    active.upstream_output_cost = Set(values.upstream_output_cost);
    active.upstream_cache_creation_cost = Set(values.upstream_cache_creation_cost);
    active.upstream_cache_read_cost = Set(values.upstream_cache_read_cost);
    active.upstream_total_cost = Set(values.upstream_total_cost);
}

pub(super) fn apply_candidate_values(active: &mut request_candidates::ActiveModel, values: RequestUpstreamCost) {
    active.upstream_cost_mode = Set(values.upstream_cost_mode.map(|value| mode_value(&value).to_owned()));
    active.upstream_cost_source = Set(values.upstream_cost_source.map(|value| source_value(&value).to_owned()));
    active.upstream_price_per_request = Set(values.upstream_price_per_request);
    active.upstream_input_price_per_million = Set(values.upstream_input_price_per_million);
    active.upstream_output_price_per_million = Set(values.upstream_output_price_per_million);
    active.upstream_cache_creation_price_per_million = Set(values.upstream_cache_creation_price_per_million);
    active.upstream_cache_read_price_per_million = Set(values.upstream_cache_read_price_per_million);
    active.upstream_request_cost = Set(values.upstream_request_cost);
    active.upstream_input_cost = Set(values.upstream_input_cost);
    active.upstream_output_cost = Set(values.upstream_output_cost);
    active.upstream_cache_creation_cost = Set(values.upstream_cache_creation_cost);
    active.upstream_cache_read_cost = Set(values.upstream_cache_read_cost);
    active.upstream_total_cost = Set(values.upstream_total_cost);
}

pub(super) fn apply_request_patch(active: &mut request_records::ActiveModel, patch: RequestUpstreamCostRecordPatch) {
    apply_mode_patch(&mut active.upstream_cost_mode, patch.upstream_cost_mode);
    apply_source_patch(&mut active.upstream_cost_source, patch.upstream_cost_source);
    apply_decimal_patch(&mut active.upstream_price_per_request, patch.upstream_price_per_request);
    apply_decimal_patch(&mut active.upstream_input_price_per_million, patch.upstream_input_price_per_million);
    apply_decimal_patch(&mut active.upstream_output_price_per_million, patch.upstream_output_price_per_million);
    apply_decimal_patch(&mut active.upstream_cache_creation_price_per_million, patch.upstream_cache_creation_price_per_million);
    apply_decimal_patch(&mut active.upstream_cache_read_price_per_million, patch.upstream_cache_read_price_per_million);
    apply_decimal_patch(&mut active.upstream_request_cost, patch.upstream_request_cost);
    apply_decimal_patch(&mut active.upstream_input_cost, patch.upstream_input_cost);
    apply_decimal_patch(&mut active.upstream_output_cost, patch.upstream_output_cost);
    apply_decimal_patch(&mut active.upstream_cache_creation_cost, patch.upstream_cache_creation_cost);
    apply_decimal_patch(&mut active.upstream_cache_read_cost, patch.upstream_cache_read_cost);
    apply_decimal_patch(&mut active.upstream_total_cost, patch.upstream_total_cost);
}

pub(super) fn apply_candidate_patch(active: &mut request_candidates::ActiveModel, patch: RequestUpstreamCostRecordPatch) {
    apply_mode_patch(&mut active.upstream_cost_mode, patch.upstream_cost_mode);
    apply_source_patch(&mut active.upstream_cost_source, patch.upstream_cost_source);
    apply_decimal_patch(&mut active.upstream_price_per_request, patch.upstream_price_per_request);
    apply_decimal_patch(&mut active.upstream_input_price_per_million, patch.upstream_input_price_per_million);
    apply_decimal_patch(&mut active.upstream_output_price_per_million, patch.upstream_output_price_per_million);
    apply_decimal_patch(&mut active.upstream_cache_creation_price_per_million, patch.upstream_cache_creation_price_per_million);
    apply_decimal_patch(&mut active.upstream_cache_read_price_per_million, patch.upstream_cache_read_price_per_million);
    apply_decimal_patch(&mut active.upstream_request_cost, patch.upstream_request_cost);
    apply_decimal_patch(&mut active.upstream_input_cost, patch.upstream_input_cost);
    apply_decimal_patch(&mut active.upstream_output_cost, patch.upstream_output_cost);
    apply_decimal_patch(&mut active.upstream_cache_creation_cost, patch.upstream_cache_creation_cost);
    apply_decimal_patch(&mut active.upstream_cache_read_cost, patch.upstream_cache_read_cost);
    apply_decimal_patch(&mut active.upstream_total_cost, patch.upstream_total_cost);
}

pub(super) fn mode_value(value: &ProviderModelCostMode) -> &'static str {
    match value {
        ProviderModelCostMode::PerRequest => "per_request",
        ProviderModelCostMode::PerToken => "per_token",
    }
}

pub(super) fn source_value(value: &ProviderModelCostSource) -> &'static str {
    match value {
        ProviderModelCostSource::Configured => "configured",
        ProviderModelCostSource::GlobalDefault => "global_default",
    }
}

fn parse_mode(value: Option<String>) -> StorageResult<Option<ProviderModelCostMode>> {
    value.map(|value| mode_from_str(&value)).transpose()
}

fn parse_source(value: Option<String>) -> StorageResult<Option<ProviderModelCostSource>> {
    value.map(|value| source_from_str(&value)).transpose()
}

fn apply_mode_patch(active: &mut ActiveValue<Option<String>>, patch: PatchField<ProviderModelCostMode>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(mode_value(&value).to_owned())),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_source_patch(active: &mut ActiveValue<Option<String>>, patch: PatchField<ProviderModelCostSource>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(source_value(&value).to_owned())),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn apply_decimal_patch(active: &mut ActiveValue<Option<Decimal>>, patch: PatchField<Decimal>) {
    match patch {
        PatchField::Value(value) => *active = Set(Some(value)),
        PatchField::Null => *active = Set(None),
        PatchField::Missing => {}
    }
}

fn mode_from_str(value: &str) -> StorageResult<ProviderModelCostMode> {
    match value {
        "per_request" => Ok(ProviderModelCostMode::PerRequest),
        "per_token" => Ok(ProviderModelCostMode::PerToken),
        other => Err(StorageError::Database(format!("invalid upstream cost mode: {other}"))),
    }
}

fn source_from_str(value: &str) -> StorageResult<ProviderModelCostSource> {
    match value {
        "configured" => Ok(ProviderModelCostSource::Configured),
        "global_default" => Ok(ProviderModelCostSource::GlobalDefault),
        other => Err(StorageError::Database(format!("invalid upstream cost source: {other}"))),
    }
}
