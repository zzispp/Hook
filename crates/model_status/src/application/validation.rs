use types::model_status::{
    ModelStatusCheckBatchCreateRequest, ModelStatusCheckBatchUpdateRequest, ModelStatusCheckCreate, ModelStatusCheckUpdate, ModelStatusRunListRequest,
};

use super::{ModelStatusDispatchOptions, ModelStatusError, ModelStatusResult};

const MIN_INTERVAL_SECONDS: i64 = 60;
const MAX_INTERVAL_SECONDS: i64 = 3600;
const MAX_BATCH_OPERATION: usize = 100;
const MAX_RUN_PAGE_SIZE: u64 = 100;

pub fn validate_create(input: &ModelStatusCheckCreate) -> ModelStatusResult<()> {
    validate_required("name", &input.name)?;
    validate_required("global_model_id", &input.global_model_id)?;
    validate_required("api_format", &input.api_format)?;
    validate_required("api_token_id", &input.api_token_id)?;
    validate_interval(input.interval_seconds)
}

pub fn validate_batch_create(input: &ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<()> {
    validate_required("name_prefix", &input.name_prefix)?;
    validate_required("api_token_id", &input.api_token_id)?;
    validate_batch_ids(&input.global_model_ids)?;
    validate_api_formats(&input.api_formats)?;
    validate_batch_size(input.global_model_ids.len() * input.api_formats.len())?;
    validate_interval(input.interval_seconds)
}

pub fn validate_update(input: &ModelStatusCheckUpdate) -> ModelStatusResult<()> {
    if input.name.is_none()
        && input.global_model_id.is_none()
        && input.api_format.is_none()
        && input.api_token_id.is_none()
        && input.interval_seconds.is_none()
        && input.enabled.is_none()
    {
        return Err(ModelStatusError::InvalidInput("update payload is empty".into()));
    }
    if let Some(value) = input.interval_seconds {
        validate_interval(value)?;
    }
    Ok(())
}

pub fn validate_batch_delete(ids: &[String]) -> ModelStatusResult<()> {
    validate_batch_ids(ids)
}

pub fn validate_batch_update(input: &ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<()> {
    validate_batch_ids(&input.ids)?;
    validate_batch_update_patch(input)?;
    if let Some(value) = input.interval_seconds {
        validate_interval(value)?;
    }
    if let Some(value) = input.name_prefix.as_deref() {
        validate_required("name_prefix", value)?;
    }
    if let Some(value) = input.api_token_id.as_deref() {
        validate_required("api_token_id", value)?;
    }
    Ok(())
}

pub fn validate_run_list(request: &ModelStatusRunListRequest) -> ModelStatusResult<()> {
    if request.page_size == 0 || request.page_size > MAX_RUN_PAGE_SIZE {
        return Err(ModelStatusError::InvalidInput(format!("page_size must be between 1 and {MAX_RUN_PAGE_SIZE}")));
    }
    Ok(())
}

pub fn validate_dispatch_options(options: ModelStatusDispatchOptions) -> ModelStatusResult<()> {
    if options.limit == 0 {
        return Err(ModelStatusError::InvalidInput("batch_size must be greater than 0".into()));
    }
    if options.concurrency == 0 {
        return Err(ModelStatusError::InvalidInput("concurrency must be greater than 0".into()));
    }
    if options.provider_key_min_interval_seconds <= 0 {
        return Err(ModelStatusError::InvalidInput(
            "provider_key_min_interval_seconds must be greater than 0".into(),
        ));
    }
    Ok(())
}

fn validate_interval(value: i64) -> ModelStatusResult<()> {
    if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&value) {
        return Err(ModelStatusError::InvalidInput(format!(
            "interval_seconds must be between {MIN_INTERVAL_SECONDS} and {MAX_INTERVAL_SECONDS}"
        )));
    }
    Ok(())
}

fn validate_required(field: &str, value: &str) -> ModelStatusResult<()> {
    if value.trim().is_empty() {
        return Err(ModelStatusError::InvalidInput(format!("{field} is required")));
    }
    Ok(())
}

fn validate_batch_ids(ids: &[String]) -> ModelStatusResult<()> {
    if ids.is_empty() || ids.len() > MAX_BATCH_OPERATION {
        return Err(ModelStatusError::InvalidInput(format!(
            "ids length must be between 1 and {MAX_BATCH_OPERATION}"
        )));
    }
    if ids.iter().any(|id| id.trim().is_empty()) {
        return Err(ModelStatusError::InvalidInput("ids cannot contain blank values".into()));
    }
    Ok(())
}

fn validate_batch_update_patch(input: &ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<()> {
    if input.enabled.is_none() && input.interval_seconds.is_none() && input.name_prefix.is_none() && input.api_token_id.is_none() {
        return Err(ModelStatusError::InvalidInput("batch update payload is empty".into()));
    }
    Ok(())
}

fn validate_api_formats(values: &[String]) -> ModelStatusResult<()> {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        return Err(ModelStatusError::InvalidInput("api_formats cannot be empty or contain blank values".into()));
    }
    Ok(())
}

fn validate_batch_size(value: usize) -> ModelStatusResult<()> {
    if value == 0 || value > MAX_BATCH_OPERATION {
        return Err(ModelStatusError::InvalidInput(format!(
            "batch size must be between 1 and {MAX_BATCH_OPERATION}"
        )));
    }
    Ok(())
}
