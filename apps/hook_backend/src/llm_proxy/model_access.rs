use types::api_token::ApiToken;

use super::{
    cache::snapshot::{CachedBillingGroup, CachedGlobalModel, CachedProvider, CachedUserAccess, SchedulingSnapshot},
    client_error, LlmProxyError,
};

pub(crate) fn visible_models_for_token<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<Vec<&'a CachedGlobalModel>, LlmProxyError> {
    let token_user = token_user_for_snapshot(snapshot, token)?;
    let user_access = user_access_for_token(token, token_user);
    let group = active_group(snapshot, token, token_user)?;
    Ok(snapshot
        .models
        .iter()
        .filter(|model| model.is_active)
        .filter(|model| ensure_token_allows_model(token, &model.id).is_ok())
        .filter(|model| ensure_group_allows_model(group, &model.id).is_ok())
        .filter(|model| ensure_user_allows_model(user_access, &model.id).is_ok())
        .collect())
}

pub(crate) fn visible_model_for_token<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken, value: &str) -> Result<&'a CachedGlobalModel, LlmProxyError> {
    visible_models_for_token(snapshot, token)?
        .into_iter()
        .find(|model| model.id == value || model.name == value)
        .ok_or_else(|| LlmProxyError::NotFound(format!("model not found: {value}")))
}

pub(crate) fn active_group<'a>(
    snapshot: &'a SchedulingSnapshot,
    token: &ApiToken,
    token_user: Option<&CachedUserAccess>,
) -> Result<&'a CachedBillingGroup, LlmProxyError> {
    let group = snapshot
        .groups
        .iter()
        .find(|group| group.code == token.group_code)
        .ok_or_else(|| LlmProxyError::Forbidden(format!("billing group not found: {}", token.group_code)))?;
    if !group.is_active {
        return Err(LlmProxyError::Forbidden(format!("billing group is inactive: {}", group.code)));
    }
    ensure_group_visible_to_token_owner(snapshot, group, token_user)?;
    Ok(group)
}

fn ensure_group_visible_to_token_owner(
    snapshot: &SchedulingSnapshot,
    group: &CachedBillingGroup,
    token_user: Option<&CachedUserAccess>,
) -> Result<(), LlmProxyError> {
    let Some(user) = token_user else {
        return Ok(());
    };
    if !snapshot.active_user_group_codes.iter().any(|code| code == &user.group_code) {
        return Err(LlmProxyError::Forbidden(format!("user group is inactive or unavailable: {}", user.group_code)));
    }
    if group.visible_user_group_codes.iter().any(|code| code == &user.group_code) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!(
        "billing group is not visible to user group {}: {}",
        user.group_code, group.code
    )))
}

pub(crate) fn ensure_token_allows_model(token: &ApiToken, model_id: &str) -> Result<(), LlmProxyError> {
    if token.model_access_mode == types::api_token::ModelAccessMode::All || token.allowed_model_ids.iter().any(|id| id == model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by token: {model_id}")))
}

pub(crate) fn ensure_group_allows_model(group: &CachedBillingGroup, model_id: &str) -> Result<(), LlmProxyError> {
    if ids_allow(&group.allowed_model_ids, model_id) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!(
        "model is not allowed by billing group {}: {model_id}",
        group.code
    )))
}

pub(crate) fn ensure_user_allows_model(access: Option<&CachedUserAccess>, model_id: &str) -> Result<(), LlmProxyError> {
    if access.is_none_or(|access| ids_allow(&access.allowed_model_ids, model_id)) {
        return Ok(());
    }
    Err(LlmProxyError::Forbidden(format!("model is not allowed by user: {model_id}")))
}

pub(crate) fn provider_allowed(group: &CachedBillingGroup, user_access: Option<&CachedUserAccess>, provider: &CachedProvider) -> bool {
    provider.is_active
        && ids_allow(&group.allowed_provider_ids, &provider.id)
        && user_access.is_none_or(|access| ids_allow(&access.allowed_provider_ids, &provider.id))
}

pub(crate) fn token_user_for_snapshot<'a>(snapshot: &'a SchedulingSnapshot, token: &ApiToken) -> Result<Option<&'a CachedUserAccess>, LlmProxyError> {
    let Some(user_id) = token.user_id.as_ref() else {
        if token.token_type == types::api_token::ApiTokenType::User {
            return Err(LlmProxyError::Forbidden(format!("user token missing user id: {}", token.id)));
        }
        return Ok(None);
    };
    let user = snapshot.users.iter().find(|user| user.id == *user_id);
    if token.token_type == types::api_token::ApiTokenType::User && user.is_none() {
        return Err(LlmProxyError::hook_api_forbidden(
            "user is disabled or unavailable",
            client_error::HOOK_API_ERROR_TYPE,
        ));
    }
    if token.token_type == types::api_token::ApiTokenType::User && user.is_some_and(|user| !user.is_active) {
        return Err(LlmProxyError::hook_api_forbidden(
            "user is disabled or unavailable",
            client_error::HOOK_API_ERROR_TYPE,
        ));
    }
    Ok(user)
}

pub(crate) fn user_access_for_token<'a>(token: &ApiToken, token_user: Option<&'a CachedUserAccess>) -> Option<&'a CachedUserAccess> {
    if token.token_type != types::api_token::ApiTokenType::User {
        return None;
    }
    token_user
}

pub(crate) fn ids_allow(ids: &[String], id: &str) -> bool {
    ids.is_empty() || ids.iter().any(|item| item == id)
}

#[cfg(test)]
mod tests;
