use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use types::user::{IdentityProvider, NewUser, USER_QUOTA_MODE_WALLET, User, UserIdentityInput};

use crate::application::{ReplaceUserRecord, WalletProviderSettings};

pub(in crate::application::service::social_auth) const TICKET_BYTES: usize = 32;
pub(in crate::application::service::social_auth) const OAUTH_STATE_TTL_SECONDS: u64 = 10 * 60;
pub(in crate::application::service::social_auth) const OAUTH_BINDING_TTL_SECONDS: u64 = 10 * 60;
pub(in crate::application::service::social_auth) const WALLET_CHALLENGE_TTL_SECONDS: u64 = 10 * 60;
pub(in crate::application::service::social_auth) const WALLET_BINDING_TTL_SECONDS: u64 = 10 * 60;

const NONCE_BYTES: usize = 16;

pub(in crate::application::service::social_auth) fn random_token(bytes_len: usize) -> String {
    let mut bytes = vec![0_u8; bytes_len];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub(in crate::application::service::social_auth) fn random_nonce() -> String {
    random_token(NONCE_BYTES).chars().take(16).collect()
}

pub(in crate::application::service::social_auth) fn provider_identity(
    provider: IdentityProvider,
    subject: String,
    email: Option<String>,
    email_verified: bool,
    display_name: Option<String>,
    avatar_url: Option<String>,
    metadata_json: String,
) -> UserIdentityInput {
    UserIdentityInput {
        user_id: String::new(),
        provider,
        provider_subject: subject,
        email,
        email_verified,
        display_name,
        avatar_url,
        metadata_json,
    }
}

pub(in crate::application::service::social_auth) fn new_provider_user(email: &str, default_group_code: &str) -> NewUser {
    NewUser {
        username: username_from_email(email),
        password: random_token(16),
        email: email.to_owned(),
        role: constants::auth::DEFAULT_USER_ROLE.into(),
        group_codes: Some(vec![default_group_code.to_owned()]),
        is_active: constants::auth::DEFAULT_USER_IS_ACTIVE,
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_mode: USER_QUOTA_MODE_WALLET.into(),
    }
}

pub(in crate::application::service::social_auth) fn username_from_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or("user");
    let normalized = local.chars().map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' }).collect::<String>();
    let hash = hex::encode(Sha256::digest(email.as_bytes()));
    format!("{}_{}", normalized.trim_matches('_'), &hash[..8])
}

pub(in crate::application::service::social_auth) fn normalize_subject(provider: IdentityProvider, value: &str) -> String {
    match provider {
        IdentityProvider::Evm => value.trim().to_ascii_lowercase(),
        IdentityProvider::Solana => value.trim().to_owned(),
        _ => value.trim().to_owned(),
    }
}

pub(in crate::application::service::social_auth) fn ensure_wallet_scope(
    settings: &WalletProviderSettings,
    provider: IdentityProvider,
    chain_id: Option<u64>,
    network: Option<&str>,
) -> crate::application::AppResult<()> {
    match provider {
        IdentityProvider::Evm if chain_id.is_some_and(|id| settings.evm_chain_ids.contains(&id)) => Ok(()),
        IdentityProvider::Solana if network == Some(settings.solana_network.as_str()) => Ok(()),
        IdentityProvider::Evm => Err(crate::application::AppError::InvalidInput("EVM chain is not allowed".into())),
        IdentityProvider::Solana => Err(crate::application::AppError::InvalidInput("Solana network is not allowed".into())),
        _ => Err(crate::application::AppError::InvalidInput("wallet provider is invalid".into())),
    }
}

pub(in crate::application::service::social_auth) fn password_replace_record(user: User, password_hash: String) -> ReplaceUserRecord {
    ReplaceUserRecord {
        username: user.username,
        password_hash: Some(password_hash),
        email: user.email,
        email_verified: None,
        role: user.role,
        group_codes: user.group_codes,
        is_active: user.is_active,
        allowed_model_ids: user.allowed_model_ids,
        allowed_provider_ids: user.allowed_provider_ids,
        rate_limit_rpm: user.rate_limit_rpm,
        quota_mode: user.quota_mode,
    }
}
