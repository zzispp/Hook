use types::model::{GlobalModelCreate, GlobalModelListRequest, GlobalModelUpdate, PatchField, PricingTier, TieredPricingConfig};

use super::{ModelError, ModelResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_BATCH_DELETE: usize = 100;
const MAX_LIST_LIMIT: u64 = 1000;

pub fn sanitize_create(input: GlobalModelCreate) -> GlobalModelCreate {
    GlobalModelCreate {
        name: input.name.trim().to_owned(),
        display_name: input.display_name.trim().to_owned(),
        ..input
    }
}

pub fn sanitize_update(input: GlobalModelUpdate) -> GlobalModelUpdate {
    GlobalModelUpdate {
        display_name: input.display_name.map(|value| value.trim().to_owned()),
        ..input
    }
}

pub fn validate_create(input: &GlobalModelCreate) -> ModelResult<()> {
    validate_name("name", &input.name)?;
    validate_name("display_name", &input.display_name)?;
    validate_tiered_pricing(&input.default_tiered_pricing)
}

pub fn validate_update(input: &GlobalModelUpdate) -> ModelResult<()> {
    if input.is_empty() {
        return Err(ModelError::InvalidInput("update payload is empty".into()));
    }
    validate_optional_display_name(input.display_name.as_deref())?;
    match &input.default_tiered_pricing {
        PatchField::Value(pricing) => validate_tiered_pricing(pricing),
        PatchField::Null => Err(ModelError::InvalidInput("default_tiered_pricing cannot be null".into())),
        PatchField::Missing => Ok(()),
    }
}

pub fn validate_list_request(request: &GlobalModelListRequest) -> ModelResult<()> {
    if request.limit == 0 || request.limit > MAX_LIST_LIMIT {
        return Err(ModelError::InvalidInput(format!("limit must be between 1 and {MAX_LIST_LIMIT}")));
    }
    Ok(())
}

pub fn validate_batch_delete(ids: &[String]) -> ModelResult<()> {
    if ids.is_empty() || ids.len() > MAX_BATCH_DELETE {
        return Err(ModelError::InvalidInput(format!("ids length must be between 1 and {MAX_BATCH_DELETE}")));
    }
    if ids.iter().any(|id| id.trim().is_empty()) {
        return Err(ModelError::InvalidInput("ids cannot contain blank values".into()));
    }
    Ok(())
}

fn validate_optional_display_name(value: Option<&str>) -> ModelResult<()> {
    match value {
        Some(value) => validate_name("display_name", value),
        None => Ok(()),
    }
}

fn validate_name(field: &str, value: &str) -> ModelResult<()> {
    if value.is_empty() || value.len() > MAX_NAME_LENGTH {
        return Err(ModelError::InvalidInput(format!("{field} length must be between 1 and {MAX_NAME_LENGTH}")));
    }
    Ok(())
}

fn validate_tiered_pricing(config: &TieredPricingConfig) -> ModelResult<()> {
    if config.tiers.is_empty() {
        return Err(ModelError::InvalidInput("default_tiered_pricing.tiers cannot be empty".into()));
    }
    validate_tier_order(&config.tiers)
}

fn validate_tier_order(tiers: &[PricingTier]) -> ModelResult<()> {
    let mut previous = 0;
    let mut unlimited_seen = false;
    for (index, tier) in tiers.iter().enumerate() {
        validate_tier(index, tier, previous, unlimited_seen)?;
        if tier.up_to.is_none() {
            unlimited_seen = true;
        } else if let Some(up_to) = tier.up_to {
            previous = up_to;
        }
    }
    if unlimited_seen {
        return Ok(());
    }
    Err(ModelError::InvalidInput("last pricing tier must have up_to=null".into()))
}

fn validate_tier(index: usize, tier: &PricingTier, previous: u64, unlimited_seen: bool) -> ModelResult<()> {
    if unlimited_seen {
        return Err(ModelError::InvalidInput("up_to=null tier must be the last tier".into()));
    }
    if tier.up_to.is_some_and(|up_to| up_to <= previous) {
        return Err(ModelError::InvalidInput(format!(
            "pricing tier {} up_to must be greater than previous tier",
            index + 1
        )));
    }
    Ok(())
}
