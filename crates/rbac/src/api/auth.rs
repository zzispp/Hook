use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use types::user::User;
use user::application::AppError;

use crate::{
    api::{RbacApiError, state::RbacApiState},
    application::{ApiCheckRequest, RbacError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub role: String,
    pub system: bool,
}

pub async fn auth_middleware(State(state): State<RbacApiState>, mut request: Request, next: Next) -> Result<Response, RbacApiError> {
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    if state.rbac.is_whitelisted(&state.authorization, &method, &path)? {
        return Ok(next.run(request).await);
    }

    let token = bearer_token(request.headers())?;
    let user_id = state.tokens.validate_access(token).map_err(|_| RbacError::Unauthorized)?;
    let user = state.users.authenticated_user(user_id).await.map_err(rbac_user_error)?;
    let current_user = current_user(user);
    state
        .rbac
        .authorize_api(&state.authorization, user_check_request(&method, &path, &current_user))
        .await?;
    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

fn user_check_request(method: &str, path: &str, current_user: &CurrentUser) -> ApiCheckRequest {
    ApiCheckRequest {
        method: method.into(),
        path: path.into(),
        role_code: current_user.role.clone(),
        system: current_user.system,
    }
}

fn current_user(user: User) -> CurrentUser {
    CurrentUser {
        system: user.system,
        id: user.id.0,
        username: user.username,
        role: user.role,
    }
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, RbacError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(RbacError::Unauthorized)?;
    value.strip_prefix("Bearer ").ok_or(RbacError::Unauthorized)
}

fn rbac_user_error(error: AppError) -> RbacError {
    match error {
        AppError::Infrastructure(message) => RbacError::Infrastructure(message),
        _ => RbacError::Unauthorized,
    }
}
