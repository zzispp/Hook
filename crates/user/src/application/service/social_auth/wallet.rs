use types::user::{IdentityProvider, UserId, UserIdentitySummary};

use crate::application::{
    AppError, AppResult, AuthProviderConfig, AuthTicketStore, UserRepository, WalletChallenge, WalletProviderSettings, WalletSignInInput, WalletSignInResult,
};

use super::helpers::{WALLET_CHALLENGE_TTL_SECONDS, ensure_wallet_scope, normalize_subject, provider_identity, random_nonce};

pub(in crate::application::service) async fn wallet_nonce<C, T>(
    config: &C,
    tickets: &T,
    provider: IdentityProvider,
    address: String,
    chain_id: Option<u64>,
) -> AppResult<WalletChallenge>
where
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = config.wallet_provider_settings().await?;
    ensure_wallet_provider_enabled(&settings, provider)?;
    ensure_wallet_scope(&settings, provider, chain_id)?;
    let nonce = random_nonce();
    let message = wallet_message(&settings, provider, &address, chain_id, &nonce)?;
    let challenge = WalletChallenge {
        provider,
        address: normalize_subject(provider, &address),
        nonce,
        message,
        chain_id,
    };
    tickets
        .save_wallet_challenge(&challenge.nonce, challenge.clone(), WALLET_CHALLENGE_TTL_SECONDS)
        .await?;
    Ok(challenge)
}

pub(in crate::application::service) struct WalletSignInDeps<'a, R, C, T> {
    pub repository: &'a R,
    pub config: &'a C,
    pub tickets: &'a T,
}

pub(in crate::application::service) async fn wallet_sign_in<R, C, T>(
    deps: WalletSignInDeps<'_, R, C, T>,
    input: WalletSignInInput,
) -> AppResult<WalletSignInResult>
where
    R: UserRepository,
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = deps.config.wallet_provider_settings().await?;
    ensure_wallet_provider_enabled(&settings, input.provider)?;
    let nonce = message_nonce(input.provider, &input.message)?;
    let challenge = deps.tickets.consume_wallet_challenge(&nonce).await?.ok_or(AppError::Unauthorized)?;
    if challenge.chain_id != input.chain_id {
        return Err(AppError::Unauthorized);
    }
    verify_wallet_challenge(&settings, &challenge, input.provider, &input.address, &input.message, &input.signature).await?;
    existing_wallet_identity_or_account_required(deps.repository, input.provider, &input.address).await
}

pub(in crate::application::service) async fn account_wallet_link<R, C, T>(
    deps: WalletSignInDeps<'_, R, C, T>,
    user_id: UserId,
    input: WalletSignInInput,
) -> AppResult<UserIdentitySummary>
where
    R: UserRepository,
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = deps.config.wallet_provider_settings().await?;
    ensure_wallet_provider_enabled(&settings, input.provider)?;
    let nonce = message_nonce(input.provider, &input.message)?;
    let challenge = deps.tickets.consume_wallet_challenge(&nonce).await?.ok_or(AppError::Unauthorized)?;
    if challenge.chain_id != input.chain_id {
        return Err(AppError::Unauthorized);
    }
    verify_wallet_challenge(&settings, &challenge, input.provider, &input.address, &input.message, &input.signature).await?;
    let user = deps.repository.find_by_id(user_id).await?.ok_or(AppError::NotFound)?;
    let subject = normalize_subject(input.provider, &input.address);
    if deps.repository.find_identity(input.provider, &subject).await?.is_some() {
        return Err(AppError::InvalidInput("provider identity is already linked".into()));
    }
    let identity = types::user::UserIdentityInput {
        user_id: user.id.0,
        email: Some(user.email),
        email_verified: user.email_verified,
        ..wallet_identity(input.provider, subject)
    };
    Ok(UserIdentitySummary::from(deps.repository.create_identity(identity).await?))
}

async fn existing_wallet_identity_or_account_required<R>(repository: &R, provider: IdentityProvider, address: &str) -> AppResult<WalletSignInResult>
where
    R: UserRepository,
{
    let subject = normalize_subject(provider, address);
    if let Some(identity) = repository.find_identity(provider, &subject).await? {
        repository.touch_identity_login(&identity.id).await?;
        let user = repository.find_by_id(UserId(identity.user_id)).await?.ok_or(AppError::NotFound)?;
        repository.record_login(user.id.clone()).await?;
        return Ok(WalletSignInResult::Authenticated(Box::new(user)));
    }

    Ok(WalletSignInResult::AccountRequired { provider, address: subject })
}

fn wallet_identity(provider: IdentityProvider, subject: String) -> types::user::UserIdentityInput {
    provider_identity(provider, subject, None, false, None, None, "{}".into())
}

fn wallet_message(settings: &WalletProviderSettings, provider: IdentityProvider, address: &str, chain_id: Option<u64>, nonce: &str) -> AppResult<String> {
    let issued_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("current time must format");
    match provider {
        IdentityProvider::Evm => Ok(evm_message(settings, address, chain_id, nonce, &issued_at)),
        _ => Err(AppError::InvalidInput("wallet provider is invalid".into())),
    }
}

fn evm_message(settings: &WalletProviderSettings, address: &str, chain_id: Option<u64>, nonce: &str, issued_at: &str) -> String {
    format!(
        "{} wants you to sign in with your Ethereum account:\n{}\n\n{}\n\nURI: https://{}\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}",
        settings.domain,
        address,
        settings.evm_statement,
        settings.domain,
        chain_id.unwrap_or(1),
        nonce,
        issued_at,
    )
}

async fn verify_wallet_challenge(
    settings: &WalletProviderSettings,
    challenge: &WalletChallenge,
    provider: IdentityProvider,
    address: &str,
    message: &str,
    signature: &str,
) -> AppResult<()> {
    if challenge.provider != provider || challenge.address != normalize_subject(provider, address) || challenge.message != message {
        return Err(AppError::Unauthorized);
    }
    ensure_wallet_scope(settings, provider, challenge.chain_id)?;
    match provider {
        IdentityProvider::Evm => verify_evm_message(settings, challenge, message, signature).await,
        _ => Err(AppError::InvalidInput("wallet provider is invalid".into())),
    }
}

async fn verify_evm_message(settings: &WalletProviderSettings, challenge: &WalletChallenge, message: &str, signature: &str) -> AppResult<()> {
    let message: siwe::Message = message.parse().map_err(|_| AppError::Unauthorized)?;
    if message.chain_id != challenge.chain_id.unwrap_or(1) {
        return Err(AppError::Unauthorized);
    }
    let signature = decode_hex_signature(signature)?;
    message
        .verify(
            &signature,
            &siwe::VerificationOpts {
                domain: Some(settings.domain.parse().map_err(|_| AppError::InvalidInput("wallet domain is invalid".into()))?),
                nonce: Some(challenge.nonce.clone()),
                timestamp: Some(time::OffsetDateTime::now_utc()),
            },
        )
        .await
        .map_err(|_| AppError::Unauthorized)
}

fn decode_hex_signature(signature: &str) -> AppResult<Vec<u8>> {
    let trimmed = signature.trim().trim_start_matches("0x");
    hex::decode(trimmed).map_err(|_| AppError::Unauthorized)
}

fn ensure_wallet_provider_enabled(settings: &WalletProviderSettings, provider: IdentityProvider) -> AppResult<()> {
    match provider {
        IdentityProvider::Evm if settings.evm_enabled => Ok(()),
        IdentityProvider::Evm => Err(AppError::InvalidInput("wallet provider is disabled".into())),
        _ => Err(AppError::InvalidInput("wallet provider is invalid".into())),
    }
}

fn message_nonce(provider: IdentityProvider, message: &str) -> AppResult<String> {
    match provider {
        IdentityProvider::Evm => {
            let message: siwe::Message = message.parse().map_err(|_| AppError::Unauthorized)?;
            Ok(message.nonce)
        }
        _ => Err(AppError::InvalidInput("wallet provider is invalid".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::MemoryUserRepository;

    #[tokio::test]
    async fn unknown_wallet_identity_requires_existing_account() {
        let repository = MemoryUserRepository::default();

        let result = existing_wallet_identity_or_account_required(&repository, IdentityProvider::Evm, "0xABC")
            .await
            .unwrap();

        assert_eq!(
            result,
            WalletSignInResult::AccountRequired {
                provider: IdentityProvider::Evm,
                address: "0xabc".into(),
            }
        );
    }
}
