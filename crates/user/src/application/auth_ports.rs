use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use types::user::{IdentityProvider, User, UserIdentityInput};

use super::AppResult;
use types::user::UserId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OAuthProviderSettings {
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
    pub public_base_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletProviderSettings {
    pub evm_enabled: bool,
    pub evm_chain_ids: Vec<u64>,
    pub evm_statement: String,
    pub domain: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OAuthStateRecord {
    pub provider: IdentityProvider,
    pub redirect_uri: String,
    pub user_id: Option<UserId>,
    pub aff_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OAuthPendingBinding {
    pub user_id: UserId,
    pub identity: UserIdentityInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthProfile {
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OAuthSignInResult {
    Authenticated(Box<User>),
    BindingRequired {
        ticket: String,
        provider: IdentityProvider,
        email: String,
        username: String,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WalletChallenge {
    pub provider: IdentityProvider,
    pub address: String,
    pub nonce: String,
    pub message: String,
    pub chain_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletNonceInput {
    pub provider: IdentityProvider,
    pub address: String,
    pub chain_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletSignInInput {
    pub provider: IdentityProvider,
    pub address: String,
    pub message: String,
    pub signature: String,
    pub chain_id: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletRegisterInput {
    pub wallet: WalletSignInInput,
    pub username: String,
    pub email: String,
    pub email_verification_code: String,
    pub aff_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalletSignInResult {
    Authenticated(Box<User>),
    AccountRequired { provider: IdentityProvider, address: String },
}

#[async_trait]
pub trait AuthProviderConfig: Send + Sync + 'static {
    async fn oauth_provider_settings(&self, provider: IdentityProvider) -> AppResult<OAuthProviderSettings>;
    async fn wallet_provider_settings(&self) -> AppResult<WalletProviderSettings>;
}

#[async_trait]
pub trait OAuthClient: Send + Sync + 'static {
    async fn fetch_profile(&self, provider: IdentityProvider, settings: OAuthProviderSettings, code: &str, redirect_uri: &str) -> AppResult<OAuthProfile>;
}

#[async_trait]
pub trait AuthTicketStore: Send + Sync + 'static {
    async fn save_oauth_state(&self, state: &str, record: OAuthStateRecord, ttl_seconds: u64) -> AppResult<()>;
    async fn consume_oauth_state(&self, state: &str) -> AppResult<Option<OAuthStateRecord>>;
    async fn save_oauth_binding(&self, ticket: &str, record: OAuthPendingBinding, ttl_seconds: u64) -> AppResult<()>;
    async fn consume_oauth_binding(&self, ticket: &str) -> AppResult<Option<OAuthPendingBinding>>;
    async fn save_wallet_challenge(&self, nonce: &str, record: WalletChallenge, ttl_seconds: u64) -> AppResult<()>;
    async fn consume_wallet_challenge(&self, nonce: &str) -> AppResult<Option<WalletChallenge>>;
}

#[async_trait]
pub trait PurposeEmailCodeStore: Send + Sync + 'static {
    async fn active_email_code(&self, purpose: &str, email: &str) -> AppResult<Option<String>>;
    async fn save_email_code(&self, purpose: &str, email: &str, code: &str, ttl_seconds: u64) -> AppResult<()>;
    async fn begin_email_code_cooldown(&self, purpose: &str, email: &str, ttl_seconds: u64) -> AppResult<bool>;
    async fn consume_email_code(&self, purpose: &str, email: &str, code: &str) -> AppResult<bool>;
}
