use types::user::{IdentityProvider, User, UserId};

use crate::application::{
    AppError, AppResult, AuthProviderConfig, AuthTicketStore, OAuthPendingBinding, OAuthProfile, OAuthProviderSettings, OAuthSignInResult, OAuthStateRecord,
    UserRepository,
};

use super::helpers::{OAUTH_BINDING_TTL_SECONDS, OAUTH_STATE_TTL_SECONDS, TICKET_BYTES, new_provider_user, provider_identity, random_token};

pub(in crate::application::service) async fn oauth_start<C, T>(config: &C, tickets: &T, provider: IdentityProvider, redirect_uri: String) -> AppResult<String>
where
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = config.oauth_provider_settings(provider).await?;
    ensure_oauth_ready(&settings)?;
    let state = random_token(TICKET_BYTES);
    tickets
        .save_oauth_state(
            &state,
            OAuthStateRecord {
                provider,
                redirect_uri: redirect_uri.clone(),
            },
            OAUTH_STATE_TTL_SECONDS,
        )
        .await?;
    Ok(oauth_authorize_url(provider, &settings.client_id, &redirect_uri, &state))
}

pub(in crate::application::service) async fn oauth_callback<R, C, T>(
    repository: &R,
    config: &C,
    tickets: &T,
    provider: IdentityProvider,
    state: &str,
    redirect_uri: &str,
    profile: OAuthProfile,
) -> AppResult<OAuthSignInResult>
where
    R: UserRepository,
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    if !profile.email_verified {
        return Err(AppError::InvalidInput("verified provider email is required".into()));
    }
    let state = tickets.consume_oauth_state(state).await?.ok_or(AppError::Unauthorized)?;
    if state.provider != provider || state.redirect_uri != redirect_uri {
        return Err(AppError::Unauthorized);
    }
    ensure_oauth_ready(&config.oauth_provider_settings(state.provider).await?)?;
    provider_profile_result(repository, tickets, state.provider, profile).await
}

pub(in crate::application::service) async fn bind_oauth_ticket<R, T>(repository: &R, tickets: &T, provider: IdentityProvider, ticket: &str) -> AppResult<User>
where
    R: UserRepository,
    T: AuthTicketStore,
{
    let pending = tickets.consume_oauth_binding(ticket).await?.ok_or(AppError::Unauthorized)?;
    if pending.identity.provider != provider {
        return Err(AppError::Unauthorized);
    }
    let user = repository.find_by_id(pending.user_id.clone()).await?.ok_or(AppError::NotFound)?;
    repository.create_identity(pending.identity).await?;
    repository.record_login(user.id.clone()).await?;
    Ok(user)
}

async fn provider_profile_result<R, T>(repository: &R, tickets: &T, provider: IdentityProvider, profile: OAuthProfile) -> AppResult<OAuthSignInResult>
where
    R: UserRepository,
    T: AuthTicketStore,
{
    if let Some(identity) = repository.find_identity(provider, &profile.subject).await? {
        repository.touch_identity_login(&identity.id).await?;
        let user = repository.find_by_id(UserId(identity.user_id)).await?.ok_or(AppError::NotFound)?;
        repository.record_login(user.id.clone()).await?;
        return Ok(OAuthSignInResult::Authenticated(user));
    }
    if let Some(user) = repository.find_by_email(&profile.email).await? {
        return oauth_binding_required(repository, tickets, user, provider, profile).await;
    }
    let user = create_provider_account(repository, &profile).await?;
    let identity = oauth_identity(provider, &profile, user.id.0.clone());
    repository.create_identity(identity).await?;
    repository.record_login(user.id.clone()).await?;
    Ok(OAuthSignInResult::Authenticated(user))
}

async fn oauth_binding_required<R, T>(
    _repository: &R,
    tickets: &T,
    user: User,
    provider: IdentityProvider,
    profile: OAuthProfile,
) -> AppResult<OAuthSignInResult>
where
    R: UserRepository,
    T: AuthTicketStore,
{
    let ticket = random_token(TICKET_BYTES);
    let identity = oauth_identity(provider, &profile, user.id.0.clone());
    tickets
        .save_oauth_binding(&ticket, OAuthPendingBinding { user_id: user.id, identity }, OAUTH_BINDING_TTL_SECONDS)
        .await?;
    Ok(OAuthSignInResult::BindingRequired {
        ticket,
        provider,
        email: profile.email,
        username: user.username,
    })
}

async fn create_provider_account<R>(repository: &R, profile: &OAuthProfile) -> AppResult<User>
where
    R: UserRepository,
{
    repository
        .create(super::super::provider_user_record(
            new_provider_user(&profile.email, constants::user_group::DEFAULT_USER_GROUP_CODE),
            Some(profile.email_verified),
        ))
        .await
}

fn oauth_identity(provider: IdentityProvider, profile: &OAuthProfile, user_id: String) -> types::user::UserIdentityInput {
    types::user::UserIdentityInput {
        user_id,
        ..provider_identity(
            provider,
            profile.subject.clone(),
            Some(profile.email.clone()),
            profile.email_verified,
            profile.display_name.clone(),
            profile.avatar_url.clone(),
            profile.metadata_json.clone(),
        )
    }
}

fn ensure_oauth_ready(settings: &OAuthProviderSettings) -> AppResult<()> {
    if !settings.enabled {
        return Err(AppError::InvalidInput("OAuth provider is disabled".into()));
    }
    if settings.client_id.is_empty() || settings.client_secret.is_empty() {
        return Err(AppError::InvalidInput("OAuth provider configuration is incomplete".into()));
    }
    Ok(())
}

fn oauth_authorize_url(provider: IdentityProvider, client_id: &str, redirect_uri: &str, state: &str) -> String {
    let base = match provider {
        IdentityProvider::Github => "https://github.com/login/oauth/authorize",
        IdentityProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        _ => "",
    };
    let scope = match provider {
        IdentityProvider::Github => "read:user user:email",
        IdentityProvider::Google => "openid email profile",
        _ => "",
    };
    format!(
        "{base}?client_id={}&redirect_uri={}&state={}&scope={}&response_type=code",
        encode_url(client_id),
        encode_url(redirect_uri),
        encode_url(state),
        encode_url(scope),
    )
}

fn encode_url(value: &str) -> String {
    value.replace(' ', "%20").replace(':', "%3A").replace('/', "%2F")
}
