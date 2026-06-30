use types::{
    system_setting::public_base_url_is_valid,
    user::{IdentityProvider, User, UserId, UserIdentitySummary},
};

use crate::application::{
    AppError, AppResult, AuthProviderConfig, AuthTicketStore, OAuthPendingBinding, OAuthProfile, OAuthProviderSettings, OAuthSignInResult, OAuthStateRecord,
    RegistrationSettings, UserRepository,
};

use super::helpers::{OAUTH_BINDING_TTL_SECONDS, OAUTH_STATE_TTL_SECONDS, TICKET_BYTES, new_provider_user, provider_identity, random_token};

pub(in crate::application::service) struct OAuthCallbackResult {
    pub result: OAuthSignInResult,
    pub created_user: Option<User>,
}

pub(in crate::application::service) struct OAuthCallbackWithCreationInput<'a> {
    pub provider: IdentityProvider,
    pub state: &'a str,
    pub redirect_uri: &'a str,
    pub profile: OAuthProfile,
}

pub(in crate::application::service) struct OAuthCallbackWithCreationDeps<'a, R, C, T> {
    pub repository: &'a R,
    pub config: &'a C,
    pub registration: &'a RegistrationSettings,
    pub tickets: &'a T,
}

pub(in crate::application::service) async fn oauth_start<C, T>(
    config: &C,
    tickets: &T,
    provider: IdentityProvider,
    aff_code: Option<String>,
) -> AppResult<String>
where
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = config.oauth_provider_settings(provider).await?;
    let redirect_uri = oauth_redirect_uri(&settings, provider)?;
    let state = random_token(TICKET_BYTES);
    tickets
        .save_oauth_state(
            &state,
            OAuthStateRecord {
                provider,
                redirect_uri: redirect_uri.clone(),
                user_id: None,
                aff_code: sanitize_aff_code(aff_code),
            },
            OAUTH_STATE_TTL_SECONDS,
        )
        .await?;
    Ok(oauth_authorize_url(provider, &settings.client_id, &redirect_uri, &state))
}

pub(in crate::application::service) async fn account_oauth_start<C, T>(
    config: &C,
    tickets: &T,
    user_id: UserId,
    provider: IdentityProvider,
) -> AppResult<String>
where
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let settings = config.oauth_provider_settings(provider).await?;
    let redirect_uri = oauth_redirect_uri(&settings, provider)?;
    let state = random_token(TICKET_BYTES);
    tickets
        .save_oauth_state(
            &state,
            OAuthStateRecord {
                provider,
                redirect_uri: redirect_uri.clone(),
                user_id: Some(user_id),
                aff_code: None,
            },
            OAUTH_STATE_TTL_SECONDS,
        )
        .await?;
    Ok(oauth_authorize_url(provider, &settings.client_id, &redirect_uri, &state))
}

pub(in crate::application::service) fn oauth_redirect_uri(settings: &OAuthProviderSettings, provider: IdentityProvider) -> AppResult<String> {
    ensure_oauth_ready(settings)?;
    Ok(format!(
        "{}/auth/oauth/callback/{}",
        settings.public_base_url.trim().trim_end_matches('/'),
        provider.as_str()
    ))
}

pub(in crate::application::service) async fn oauth_callback_with_creation<R, C, T>(
    deps: OAuthCallbackWithCreationDeps<'_, R, C, T>,
    input: OAuthCallbackWithCreationInput<'_>,
) -> AppResult<OAuthCallbackResult>
where
    R: UserRepository,
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let OAuthCallbackWithCreationDeps {
        repository,
        config,
        registration,
        tickets,
    } = deps;
    let OAuthCallbackWithCreationInput {
        provider,
        state,
        redirect_uri,
        profile,
    } = input;
    if !profile.email_verified {
        return Err(AppError::InvalidInput("verified provider email is required".into()));
    }
    let state = tickets.consume_oauth_state(state).await?.ok_or(AppError::Unauthorized)?;
    if state.provider != provider || state.redirect_uri != redirect_uri {
        return Err(AppError::Unauthorized);
    }
    ensure_oauth_ready(&config.oauth_provider_settings(state.provider).await?)?;
    if state.user_id.is_some() {
        return Err(AppError::Unauthorized);
    }
    provider_profile_result(repository, tickets, registration, state.provider, profile, state.aff_code).await
}

pub(in crate::application::service) struct AccountOAuthCallbackInput<'a> {
    pub expected_user_id: UserId,
    pub provider: IdentityProvider,
    pub state: &'a str,
    pub redirect_uri: &'a str,
    pub profile: OAuthProfile,
}

pub(in crate::application::service) async fn account_oauth_callback<R, C, T>(
    repository: &R,
    config: &C,
    tickets: &T,
    input: AccountOAuthCallbackInput<'_>,
) -> AppResult<UserIdentitySummary>
where
    R: UserRepository,
    C: AuthProviderConfig,
    T: AuthTicketStore,
{
    let AccountOAuthCallbackInput {
        expected_user_id,
        provider,
        state,
        redirect_uri,
        profile,
    } = input;
    if !profile.email_verified {
        return Err(AppError::InvalidInput("verified provider email is required".into()));
    }
    let state = tickets.consume_oauth_state(state).await?.ok_or(AppError::Unauthorized)?;
    if state.provider != provider || state.redirect_uri != redirect_uri {
        return Err(AppError::Unauthorized);
    }
    ensure_oauth_ready(&config.oauth_provider_settings(state.provider).await?)?;
    let user_id = state.user_id.ok_or(AppError::Unauthorized)?;
    if user_id != expected_user_id {
        return Err(AppError::Unauthorized);
    }
    let user = repository.find_by_id(user_id.clone()).await?.ok_or(AppError::NotFound)?;
    if repository.find_identity(provider, &profile.subject).await?.is_some() {
        return Err(AppError::InvalidInput("provider identity is already linked".into()));
    }
    let identity = repository.create_identity(oauth_identity(provider, &profile, user.id.0)).await?;
    Ok(UserIdentitySummary::from(identity))
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

async fn provider_profile_result<R, T>(
    repository: &R,
    tickets: &T,
    registration: &RegistrationSettings,
    provider: IdentityProvider,
    profile: OAuthProfile,
    aff_code: Option<String>,
) -> AppResult<OAuthCallbackResult>
where
    R: UserRepository,
    T: AuthTicketStore,
{
    if let Some(identity) = repository.find_identity(provider, &profile.subject).await? {
        repository.touch_identity_login(&identity.id).await?;
        let user = repository.find_by_id(UserId(identity.user_id)).await?.ok_or(AppError::NotFound)?;
        repository.record_login(user.id.clone()).await?;
        return Ok(OAuthCallbackResult {
            result: OAuthSignInResult::Authenticated(Box::new(user)),
            created_user: None,
        });
    }
    if let Some(user) = repository.find_by_email(&profile.email).await? {
        let result = oauth_binding_required(repository, tickets, user, provider, profile).await?;
        return Ok(OAuthCallbackResult { result, created_user: None });
    }
    let user = create_provider_account(repository, registration, &profile, aff_code).await?;
    let identity = oauth_identity(provider, &profile, user.id.0.clone());
    repository.create_identity(identity).await?;
    repository.record_login(user.id.clone()).await?;
    Ok(OAuthCallbackResult {
        result: OAuthSignInResult::Authenticated(Box::new(user.clone())),
        created_user: Some(user),
    })
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

async fn create_provider_account<R>(repository: &R, registration: &RegistrationSettings, profile: &OAuthProfile, aff_code: Option<String>) -> AppResult<User>
where
    R: UserRepository,
{
    super::super::registration::reject_closed_registration(registration)?;
    super::super::registration::reject_disallowed_registration_email(registration, &profile.email)?;
    let mut user = new_provider_user(&profile.email, &registration.default_user_group_code);
    user.referrer_aff_code = aff_code;
    repository.create(super::super::provider_user_record(user, Some(profile.email_verified))).await
}

fn sanitize_aff_code(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
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
    if settings.public_base_url.trim().is_empty() {
        return Err(AppError::InvalidInput("public_base_url is required before using OAuth provider".into()));
    }
    let is_valid = public_base_url_is_valid(settings.public_base_url.trim())
        .map_err(|error| AppError::Infrastructure(format!("invalid public_base_url validation regex: {error}")))?;
    if !is_valid {
        return Err(AppError::InvalidInput(
            "public_base_url must be a valid HTTP or HTTPS URL before using OAuth provider".into(),
        ));
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
