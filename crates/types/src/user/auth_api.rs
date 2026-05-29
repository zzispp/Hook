use serde::{Deserialize, Serialize};

use super::{IdentityProvider, UserResponse};

#[derive(Debug, Serialize)]
pub struct AuthConfigResponse {
    pub allow_registration: bool,
    pub registration_email_verification_enabled: bool,
    pub providers: AuthProviderConfigResponse,
}

#[derive(Debug, Default, Serialize)]
pub struct AuthProviderConfigResponse {
    pub github: OAuthProviderPublicConfig,
    pub google: OAuthProviderPublicConfig,
    pub evm: WalletProviderPublicConfig,
    pub solana: WalletProviderPublicConfig,
}

#[derive(Debug, Default, Serialize)]
pub struct OAuthProviderPublicConfig {
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct WalletProviderPublicConfig {
    pub enabled: bool,
    pub domain: String,
    pub statement: String,
    pub evm_chain_ids: Vec<u64>,
    pub solana_network: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthStartQuery {
    #[serde(default)]
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthStartResponse {
    pub authorization_url: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthBindExistingPayload {
    pub binding_ticket: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OAuthCallbackResponse {
    Authenticated(AuthSessionData),
    BindingRequired {
        binding_ticket: String,
        provider: IdentityProvider,
        email: String,
        username: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct WalletNoncePayload {
    pub provider: IdentityProvider,
    pub address: String,
    pub chain_id: Option<u64>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalletNonceResponse {
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletSignInPayload {
    pub provider: IdentityProvider,
    pub address: String,
    pub message: String,
    pub signature: String,
    pub chain_id: Option<u64>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WalletSignInResponse {
    Authenticated(AuthSessionData),
    EmailRequired {
        wallet_ticket: String,
        provider: IdentityProvider,
        address: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct WalletEmailCodePayload {
    pub wallet_ticket: String,
    pub email: String,
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletCompletePayload {
    pub wallet_ticket: String,
    pub email: String,
    pub email_verification_code: String,
}

#[derive(Debug, Serialize)]
pub struct AuthSessionData {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AccountProfileResponse {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct AccountPasswordEmailCodePayload {
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountPasswordChangePayload {
    pub email_verification_code: String,
    pub password: String,
}
