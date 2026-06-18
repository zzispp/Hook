use crate::BackendResult;
use axum::http::{Method, header};
use configuration::{AuthWhitelistRule as ConfigAuthRule, Settings};
use payment::RegisteredPaymentCallbackEndpoint;
use rbac::application::{AuthWhitelistRule, AuthorizationConfig};
use tower_http::cors::{Any, CorsLayer};
use user::api::TokenSettings;

const API_PATH_PREFIX: &str = "/api";

pub(crate) fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

pub(crate) fn authorization_config(settings: &Settings, payment_endpoints: &[RegisteredPaymentCallbackEndpoint]) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: whitelist_rules(settings, payment_endpoints),
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

fn whitelist_rules(settings: &Settings, payment_endpoints: &[RegisteredPaymentCallbackEndpoint]) -> Vec<AuthWhitelistRule> {
    let mut rules = auth_rules(&settings.auth.whitelist);
    rules.extend(payment_endpoints.iter().flat_map(payment_callback_rules));
    rules
}

fn payment_callback_rules(endpoint: &RegisteredPaymentCallbackEndpoint) -> Vec<AuthWhitelistRule> {
    let path_pattern = format!("{API_PATH_PREFIX}{}", endpoint.path_pattern);
    let mut rules = vec![payment_callback_rule(endpoint, path_pattern.clone())];
    if let Some(path_pattern) = trailing_slash_variant(&path_pattern) {
        rules.push(payment_callback_rule(endpoint, path_pattern));
    }
    rules
}

fn payment_callback_rule(endpoint: &RegisteredPaymentCallbackEndpoint, path_pattern: String) -> AuthWhitelistRule {
    AuthWhitelistRule {
        methods: endpoint.methods.clone(),
        path_pattern,
    }
}

fn trailing_slash_variant(path_pattern: &str) -> Option<String> {
    if path_pattern.ends_with('/') {
        return None;
    }
    Some(format!("{path_pattern}/"))
}

#[cfg(test)]
mod tests {
    use payment::{PaymentCallbackEndpointKind, RegisteredPaymentCallbackEndpoint};

    use super::payment_callback_rules;

    #[test]
    fn payment_callback_rule_includes_trailing_slash_variant() {
        let rules = payment_callback_rules(&callback_endpoint("/payment/epay/return"));

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].path_pattern, "/api/payment/epay/return");
        assert_eq!(rules[1].path_pattern, "/api/payment/epay/return/");
        assert_eq!(rules[0].methods, vec!["GET", "POST"]);
        assert_eq!(rules[1].methods, vec!["GET", "POST"]);
    }

    fn callback_endpoint(path_pattern: &str) -> RegisteredPaymentCallbackEndpoint {
        RegisteredPaymentCallbackEndpoint {
            channel_code: "epay".into(),
            kind: PaymentCallbackEndpointKind::Return,
            methods: vec!["GET".into(), "POST".into()],
            path_pattern: path_pattern.into(),
        }
    }
}
