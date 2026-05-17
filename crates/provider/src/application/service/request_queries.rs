use types::provider::{ActiveRequestRecordRequest, ProviderCooldownListRequest, RequestRecordListRequest};

use crate::application::{ProviderError, ProviderResult};

const MAX_REQUEST_RECORD_LIMIT: u64 = 100;
const MAX_PROVIDER_COOLDOWN_LIMIT: u64 = 100;

pub fn validate_request_record_list_request(request: &RequestRecordListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_REQUEST_RECORD_LIMIT {
        return Err(ProviderError::InvalidInput(format!("limit must be between 1 and {MAX_REQUEST_RECORD_LIMIT}")));
    }
    if i64::try_from(request.skip).is_err() {
        return Err(ProviderError::InvalidInput("skip exceeds PostgreSQL integer range".into()));
    }
    if invalid_type_filter(request.type_filter.as_deref()) {
        return Err(ProviderError::InvalidInput("type must be stream or non_stream".into()));
    }
    Ok(())
}

pub fn sanitize_active_request_record_request(request: ActiveRequestRecordRequest) -> ActiveRequestRecordRequest {
    let mut ids = request
        .ids
        .into_iter()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ActiveRequestRecordRequest { ids }
}

pub fn sanitize_provider_cooldown_request(request: ProviderCooldownListRequest) -> ProviderCooldownListRequest {
    ProviderCooldownListRequest {
        search: request.search.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        }),
        ..request
    }
}

pub fn validate_provider_cooldown_request(request: &ProviderCooldownListRequest) -> ProviderResult<()> {
    if request.limit == 0 || request.limit > MAX_PROVIDER_COOLDOWN_LIMIT {
        return Err(ProviderError::InvalidInput(format!(
            "limit must be between 1 and {MAX_PROVIDER_COOLDOWN_LIMIT}"
        )));
    }
    if request.status_code.is_some_and(|value| !(100..=599).contains(&value)) {
        return Err(ProviderError::InvalidInput("status_code must be between 100 and 599".into()));
    }
    Ok(())
}

fn invalid_type_filter(value: Option<&str>) -> bool {
    matches!(value.filter(|value| !value.is_empty()), Some(value) if !matches!(value, "stream" | "non_stream"))
}
