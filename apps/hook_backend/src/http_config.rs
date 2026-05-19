use crate::BackendResult;
use axum::http::{HeaderValue, Method, header};
use configuration::{AuthWhitelistRule as ConfigAuthRule, Settings};
use rbac::application::{AuthWhitelistRule, AuthorizationConfig};
use tower_http::cors::CorsLayer;
use user::api::TokenSettings;

const FRONTEND_DEV_ORIGIN: &str = "http://localhost:8082";

pub(crate) fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static(FRONTEND_DEV_ORIGIN))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

pub(crate) fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: auth_rules(&settings.auth.whitelist),
        authenticated: auth_rules(&settings.auth.authenticated),
    }
}

pub(crate) fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
        access_token_ttl_seconds: settings.jwt.access_token_ttl_seconds,
        refresh_token_ttl_seconds: settings.jwt.refresh_token_ttl_seconds,
    })
}

fn auth_rules(rules: &[ConfigAuthRule]) -> Vec<AuthWhitelistRule> {
    rules
        .iter()
        .map(|rule| AuthWhitelistRule {
            methods: rule.methods.clone(),
            path_pattern: rule.path_pattern.clone(),
        })
        .collect()
}
