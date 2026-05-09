use matchit::Router;
use types::rbac::ApiPermissionSnapshot;

use crate::application::{ApiCheckRequest, AuthWhitelistRule, AuthorizationConfig, RbacError, RbacResult};

pub(super) fn is_whitelisted(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    rules_match(&config.whitelist, method, path)
}

pub(super) fn is_authenticated_base(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    rules_match(&config.authenticated, method, path)
}

fn rules_match(rules: &[AuthWhitelistRule], method: &str, path: &str) -> RbacResult<bool> {
    let method = method.to_ascii_uppercase();
    rules.iter().try_fold(false, |matched, rule| {
        if matched || !rule.methods.iter().any(|item| item.eq_ignore_ascii_case(&method)) {
            return Ok(matched);
        }
        path_matches(&rule.path_pattern, path)
    })
}

pub(super) fn authorize_snapshot(permissions: &[ApiPermissionSnapshot], request: &ApiCheckRequest) -> RbacResult<()> {
    let permission = permissions
        .iter()
        .find(|permission| api_permission_matches(permission, request).unwrap_or(false))
        .ok_or(RbacError::Forbidden)?;
    if permission.role_codes.iter().any(|code| code == &request.role_code) {
        return Ok(());
    }
    Err(RbacError::Forbidden)
}

fn api_permission_matches(permission: &ApiPermissionSnapshot, request: &ApiCheckRequest) -> RbacResult<bool> {
    if !permission.method.eq_ignore_ascii_case(&request.method) {
        return Ok(false);
    }
    path_matches(&permission.path_pattern, &request.path)
}

fn path_matches(pattern: &str, path: &str) -> RbacResult<bool> {
    let mut router = Router::new();
    router
        .insert(pattern, ())
        .map_err(|error| RbacError::InvalidInput(format!("invalid path pattern {pattern}: {error}")))?;
    Ok(router.at(path).is_ok())
}
