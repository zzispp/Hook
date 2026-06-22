use rust_decimal::Decimal;
use types::{
    model::PatchField,
    provider::{
        ProviderCreate, ProviderListRequest, ProviderModelBindingBatchUpdate, ProviderModelBindingCreate, ProviderModelBindingUpdate,
        ProviderModelCostBatchUpsert, ProviderModelCostMode, ProviderModelCostUpsert, ProviderUpdate,
    },
};

use super::{ProviderError, ProviderResult};

mod api_key;
mod endpoint;
mod group;

pub use api_key::{sanitize_api_key, sanitize_api_key_update, validate_api_key, validate_api_key_priority_batch, validate_api_key_update};
pub use endpoint::{sanitize_endpoint, sanitize_endpoint_update, validate_endpoint, validate_endpoint_update};
pub use group::{
    sanitize_provider_key_group, sanitize_provider_key_group_list_request, sanitize_provider_key_group_update, validate_provider_key_group,
    validate_provider_key_group_list_request, validate_provider_key_group_update,
};

const MAX_LIST_LIMIT: u64 = 1000;
const MAX_NAME_LENGTH: usize = 100;
const MAX_TYPE_LENGTH: usize = 50;
const MAX_API_FORMAT_LENGTH: usize = 50;
const MAX_URL_LENGTH: usize = 500;
const MAX_MODEL_ID_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const PROVIDER_TYPES: [&str; 1] = ["custom"];

pub fn sanitize_create(input: ProviderCreate) -> ProviderCreate {
    ProviderCreate {
        name: input.name.trim().to_owned(),
        provider_type: input.provider_type.trim().to_owned(),
        ..input
    }
}

pub fn sanitize_list_request(input: ProviderListRequest) -> ProviderListRequest {
    ProviderListRequest {
        search: input.search.and_then(trim_optional),
        api_format: input.api_format.and_then(trim_optional).map(|value| value.to_ascii_lowercase()),
        model_id: input.model_id.and_then(trim_optional),
        ..input
    }
}

pub fn sanitize_update(input: ProviderUpdate) -> ProviderUpdate {
    ProviderUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        provider_type: input.provider_type.map(|value| value.trim().to_owned()),
        ..input
    }
}

pub fn sanitize_model_binding(input: ProviderModelBindingCreate) -> ProviderModelBindingCreate {
    ProviderModelBindingCreate {
        global_model_id: input.global_model_id.trim().to_owned(),
        ..input
    }
}

pub fn sanitize_model_binding_batch(input: ProviderModelBindingBatchUpdate) -> ProviderModelBindingBatchUpdate {
    ProviderModelBindingBatchUpdate {
        create: input.create.into_iter().map(sanitize_model_binding).collect(),
        delete_ids: input.delete_ids.into_iter().map(|id| id.trim().to_owned()).collect(),
    }
}

pub fn sanitize_model_binding_update(input: ProviderModelBindingUpdate) -> ProviderModelBindingUpdate {
    input
}

pub fn sanitize_model_cost_batch(input: ProviderModelCostBatchUpsert) -> ProviderModelCostBatchUpsert {
    ProviderModelCostBatchUpsert {
        costs: input.costs.into_iter().map(sanitize_model_cost).collect(),
    }
}

pub fn validate_create(input: &ProviderCreate) -> ProviderResult<()> {
    validate_text("name", &input.name, MAX_NAME_LENGTH)?;
    validate_provider_type(&input.provider_type)?;
    Ok(())
}

pub fn validate_update(input: &ProviderUpdate) -> ProviderResult<()> {
    if input.is_empty() {
        return Err(ProviderError::InvalidInput("update payload is empty".into()));
    }
    if let Some(name) = input.name.as_deref() {
        validate_text("name", name, MAX_NAME_LENGTH)?;
    }
    if let Some(provider_type) = input.provider_type.as_deref() {
        validate_provider_type(provider_type)?;
    }
    Ok(())
}

pub fn validate_list_request(request: &ProviderListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(ProviderError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

pub fn validate_model_binding(input: &ProviderModelBindingCreate) -> ProviderResult<()> {
    validate_text("global_model_id", &input.global_model_id, MAX_NAME_LENGTH)
}

pub fn validate_model_binding_batch(input: &ProviderModelBindingBatchUpdate) -> ProviderResult<()> {
    if input.create.is_empty() && input.delete_ids.is_empty() {
        return Err(ProviderError::InvalidInput("model binding batch payload is empty".into()));
    }
    for binding in &input.create {
        validate_model_binding(binding)?;
    }
    for id in &input.delete_ids {
        validate_text("delete_ids", id, MAX_MODEL_ID_LENGTH)?;
    }
    Ok(())
}

pub fn validate_model_binding_update(input: &ProviderModelBindingUpdate) -> ProviderResult<()> {
    if model_binding_update_is_empty(input) {
        return Err(ProviderError::InvalidInput("model binding update payload is empty".into()));
    }
    Ok(())
}

pub fn validate_model_cost_batch(input: &ProviderModelCostBatchUpsert) -> ProviderResult<()> {
    if input.costs.is_empty() {
        return Err(ProviderError::InvalidInput("model costs cannot be empty".into()));
    }
    for cost in &input.costs {
        validate_model_cost(cost)?;
    }
    Ok(())
}

pub(super) fn validate_text(field: &str, value: &str, max_length: usize) -> ProviderResult<()> {
    if value.is_empty() || value.len() > max_length {
        return Err(ProviderError::InvalidInput(format!("{field} length must be between 1 and {max_length}")));
    }
    Ok(())
}

fn sanitize_model_cost(input: ProviderModelCostUpsert) -> ProviderModelCostUpsert {
    ProviderModelCostUpsert {
        provider_model_id: input.provider_model_id.trim().to_owned(),
        ..input
    }
}

fn validate_model_cost(input: &ProviderModelCostUpsert) -> ProviderResult<()> {
    validate_text("provider_model_id", &input.provider_model_id, MAX_MODEL_ID_LENGTH)?;
    match input.cost_mode {
        ProviderModelCostMode::PerRequest => validate_per_request_cost(input),
        ProviderModelCostMode::PerToken => validate_per_token_cost(input),
    }
}

fn validate_per_request_cost(input: &ProviderModelCostUpsert) -> ProviderResult<()> {
    validate_required_price("price_per_request", input.price_per_request)?;
    reject_token_prices(input)
}

fn validate_per_token_cost(input: &ProviderModelCostUpsert) -> ProviderResult<()> {
    if input.price_per_request.is_some() {
        return Err(ProviderError::InvalidInput("price_per_request must be empty for per_token costs".into()));
    }
    validate_required_price("input_price_per_million", input.input_price_per_million)?;
    validate_required_price("output_price_per_million", input.output_price_per_million)?;
    validate_required_price("cache_creation_price_per_million", input.cache_creation_price_per_million)?;
    validate_required_price("cache_read_price_per_million", input.cache_read_price_per_million)
}

fn reject_token_prices(input: &ProviderModelCostUpsert) -> ProviderResult<()> {
    let has_token_price = input.input_price_per_million.is_some()
        || input.output_price_per_million.is_some()
        || input.cache_creation_price_per_million.is_some()
        || input.cache_read_price_per_million.is_some();
    if has_token_price {
        return Err(ProviderError::InvalidInput("token prices must be empty for per_request costs".into()));
    }
    Ok(())
}

fn validate_required_price(field: &str, value: Option<Decimal>) -> ProviderResult<()> {
    let Some(value) = value else {
        return Err(ProviderError::InvalidInput(format!("{field} is required")));
    };
    if value < Decimal::ZERO {
        return Err(ProviderError::InvalidInput(format!("{field} cannot be negative")));
    }
    Ok(())
}

fn validate_provider_type(value: &str) -> ProviderResult<()> {
    validate_text("provider_type", value, MAX_TYPE_LENGTH)?;
    if PROVIDER_TYPES.contains(&value) {
        return Ok(());
    }
    Err(ProviderError::InvalidInput(format!(
        "provider_type must be one of {}",
        PROVIDER_TYPES.join(", ")
    )))
}

pub(super) fn trim_optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
}

pub(super) fn trim_patch(value: PatchField<String>) -> PatchField<String> {
    match value {
        PatchField::Value(value) => trim_optional(value).map(PatchField::Value).unwrap_or(PatchField::Null),
        other => other,
    }
}

fn model_binding_update_is_empty(input: &ProviderModelBindingUpdate) -> bool {
    input.is_active.is_none() && input.config.is_missing()
}
