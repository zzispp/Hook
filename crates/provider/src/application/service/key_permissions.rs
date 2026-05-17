use std::collections::HashSet;

use types::provider::ProviderModelBinding;

use crate::application::{ProviderError, ProviderRepository, ProviderResult};

pub async fn ensure_allowed_models_bound<R>(repository: &R, provider_id: &str, allowed_model_ids: &[String]) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    if allowed_model_ids.is_empty() {
        return Ok(());
    }

    let bindings = repository.list_model_bindings(provider_id).await?;
    validate_allowed_model_ids(allowed_model_ids, &bindings)
}

fn validate_allowed_model_ids(allowed_model_ids: &[String], bindings: &[ProviderModelBinding]) -> ProviderResult<()> {
    let bound_ids = bindings.iter().map(|binding| binding.global_model_id.as_str()).collect::<HashSet<_>>();
    let missing = missing_model_ids(allowed_model_ids, &bound_ids);

    if missing.is_empty() {
        return Ok(());
    }

    Err(ProviderError::InvalidInput(format!(
        "allowed_model_ids must be bound to provider: {}",
        missing.join(", ")
    )))
}

fn missing_model_ids(allowed_model_ids: &[String], bound_ids: &HashSet<&str>) -> Vec<String> {
    allowed_model_ids
        .iter()
        .filter(|model_id| !bound_ids.contains(model_id.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_allowed_model_ids_accepts_provider_bound_models() {
        let allowed = vec!["global-chat".to_owned()];
        let bindings = vec![binding("global-chat")];

        let result = validate_allowed_model_ids(&allowed, &bindings);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_allowed_model_ids_rejects_models_outside_provider_bindings() {
        let allowed = vec!["global-chat".to_owned(), "other-model".to_owned()];
        let bindings = vec![binding("global-chat")];

        let error = validate_allowed_model_ids(&allowed, &bindings).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: allowed_model_ids must be bound to provider: other-model");
    }

    fn binding(global_model_id: &str) -> ProviderModelBinding {
        ProviderModelBinding {
            id: format!("binding-{global_model_id}"),
            provider_id: "provider-1".to_owned(),
            global_model_id: global_model_id.to_owned(),
            provider_model_name: global_model_id.to_owned(),
            provider_model_mapping: None,
            is_active: true,
            price_per_request: None,
            tiered_pricing: None,
            config: None,
            created_at: "2026-05-17T00:00:00Z".to_owned(),
            updated_at: "2026-05-17T00:00:00Z".to_owned(),
        }
    }
}
