use std::collections::BTreeMap;

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    user::{
        AccountEmailVerifyPayload, AccountPasswordChangePayload, AccountPasswordEmailCodePayload, AccountProviderLinkResponse, AuthConfigResponse, Credentials,
        IdentityProvider, NewUser, PasswordResetConfirm, PasswordResetRequest, RegistrationEmailCodeRequest, ReplaceUser, SignUpUser, User, UserId,
        UserIdentitySummary, UserListFilters, UserWalletSummaryResponse,
    },
};

use crate::application::{
    AppError, AppResult, AuthProviderConfig, AuthTicketStore, InitialGrantLedger, OAuthClient, OAuthSignInResult, PasswordHasher, PasswordResetConfig,
    PasswordResetMailer, PasswordResetRepository, PurposeEmailCodeStore, RegistrationEmailCodeStore, RegistrationEmailConfig, RegistrationEmailMailer,
    RegistrationPolicy, SystemUserProvider, UserRepository, UserUseCase, UserWalletCatalog, WalletChallenge, WalletNonceInput, WalletSignInInput,
    WalletSignInResult,
};

mod helpers;

use super::{
    UserService,
    password_reset::{request_password_reset, reset_password},
    registration::{reject_closed_registration, reject_disallowed_registration_email, request_registration_email_code, verify_registration_email_code},
    social_auth::{self, ACCOUNT_PASSWORD_EMAIL_PURPOSE},
    system_user::{
        find_auth_by_identifier, list_with_system_user, reject_system_user_email, reject_system_user_id, reject_system_user_self_service, system_user_by_id,
    },
    validation::{
        sanitize_credentials, sanitize_password_reset_confirm, sanitize_password_reset_request, sanitize_registration_email_code_request,
        sanitize_replace_user, sanitize_sign_up_user, validate_credentials, validate_new_user, validate_page, validate_password_reset_confirm,
        validate_password_reset_request, validate_registration_email_code_request, validate_replace_user,
    },
};
use helpers::{grant_initial_balance, identity_summaries, unlink_identity, verify_password};

const ACCOUNT_EMAIL_CODE_DISABLED_MESSAGE: &str = "account email verification is disabled";

#[async_trait]
impl<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y> UserUseCase for UserService<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y>
where
    R: UserRepository + PasswordResetRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
    C: PasswordResetConfig,
    M: PasswordResetMailer,
    E: RegistrationEmailConfig,
    N: RegistrationEmailMailer,
    K: RegistrationEmailCodeStore,
    A: AuthProviderConfig,
    O: OAuthClient,
    T: AuthTicketStore,
    Y: PurposeEmailCodeStore,
{
    async fn auth_config(&self) -> AppResult<AuthConfigResponse> {
        self.registration_email_config.auth_config().await
    }

    async fn request_registration_email_code(&self, input: RegistrationEmailCodeRequest) -> AppResult<()> {
        let input = sanitize_registration_email_code_request(input);
        validate_registration_email_code_request(&input)?;
        request_registration_email_code(
            &self.registration_email_code_store,
            &self.registration_email_config,
            &self.registration_email_mailer,
            input,
        )
        .await
    }

    async fn sign_up(&self, input: SignUpUser) -> AppResult<User> {
        let settings = self.registration_policy.registration_settings().await?;
        reject_closed_registration(&settings)?;
        let input = sanitize_sign_up_user(SignUpUser {
            user: super::with_default_user_group(input.user, &settings.default_user_group_code),
            email_verification_code: input.email_verification_code,
        });
        validate_new_user(&input.user)?;
        reject_disallowed_registration_email(&settings, &input.user.email)?;
        verify_registration_email_code(&self.registration_email_code_store, &settings, &input).await?;
        let user = self.create_valid_user(input.user, settings.registration_email_verification_enabled).await?;
        grant_initial_balance(&self.initial_grants, &user, settings.default_user_grant).await?;
        Ok(user)
    }

    async fn sign_in(&self, input: Credentials) -> AppResult<User> {
        let input = sanitize_credentials(input);
        validate_credentials(&input)?;
        let found = find_auth_by_identifier(&self.repository, &self.system_users, &input.identifier)
            .await?
            .ok_or(AppError::InvalidCredentials)?;
        verify_password(&self.password_hasher, &input.password, &found)?;
        if !found.user.system {
            self.repository.record_login(found.user.id.clone()).await?;
        }
        Ok(found.user)
    }

    async fn oauth_start(&self, provider: IdentityProvider) -> AppResult<String> {
        social_auth::oauth_start(&self.auth_provider_config, &self.auth_ticket_store, provider).await
    }

    async fn oauth_callback(&self, provider: IdentityProvider, code: String, state: String) -> AppResult<OAuthSignInResult> {
        let settings = self.auth_provider_config.oauth_provider_settings(provider).await?;
        let redirect_uri = social_auth::oauth_redirect_uri(&settings, provider)?;
        let profile = self.oauth_client.fetch_profile(provider, settings, &code, &redirect_uri).await?;
        reject_system_user_email(&self.system_users, &profile.email)?;
        social_auth::oauth_callback(
            &self.repository,
            &self.auth_provider_config,
            &self.auth_ticket_store,
            provider,
            &state,
            &redirect_uri,
            profile,
        )
        .await
    }

    async fn bind_oauth_existing(&self, provider: IdentityProvider, ticket: String) -> AppResult<User> {
        social_auth::bind_oauth_ticket(&self.repository, &self.auth_ticket_store, provider, &ticket).await
    }

    async fn account_oauth_start(&self, id: UserId, provider: IdentityProvider) -> AppResult<String> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        social_auth::account_oauth_start(&self.auth_provider_config, &self.auth_ticket_store, user.id, provider).await
    }

    async fn account_oauth_callback(&self, id: UserId, provider: IdentityProvider, code: String, state: String) -> AppResult<AccountProviderLinkResponse> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        let settings = self.auth_provider_config.oauth_provider_settings(provider).await?;
        let redirect_uri = social_auth::oauth_redirect_uri(&settings, provider)?;
        let profile = self.oauth_client.fetch_profile(provider, settings, &code, &redirect_uri).await?;
        reject_system_user_email(&self.system_users, &profile.email)?;
        let identity = social_auth::account_oauth_callback(
            &self.repository,
            &self.auth_provider_config,
            &self.auth_ticket_store,
            social_auth::AccountOAuthCallbackInput {
                expected_user_id: user.id,
                provider,
                state: &state,
                redirect_uri: &redirect_uri,
                profile,
            },
        )
        .await?;
        Ok(AccountProviderLinkResponse { identity })
    }

    async fn wallet_nonce(&self, input: WalletNonceInput) -> AppResult<WalletChallenge> {
        social_auth::wallet_nonce(
            &self.auth_provider_config,
            &self.auth_ticket_store,
            input.provider,
            input.address,
            input.chain_id,
        )
        .await
    }

    async fn wallet_sign_in(&self, input: WalletSignInInput) -> AppResult<WalletSignInResult> {
        social_auth::wallet_sign_in(
            social_auth::WalletSignInDeps {
                repository: &self.repository,
                config: &self.auth_provider_config,
                tickets: &self.auth_ticket_store,
            },
            input,
        )
        .await
    }

    async fn account_wallet_link(&self, id: UserId, input: WalletSignInInput) -> AppResult<AccountProviderLinkResponse> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        let identity = social_auth::account_wallet_link(
            social_auth::WalletSignInDeps {
                repository: &self.repository,
                config: &self.auth_provider_config,
                tickets: &self.auth_ticket_store,
            },
            user.id,
            input,
        )
        .await?;
        Ok(AccountProviderLinkResponse { identity })
    }

    async fn request_password_reset(&self, input: PasswordResetRequest) -> AppResult<()> {
        let input = sanitize_password_reset_request(input);
        validate_password_reset_request(&input)?;
        request_password_reset(&self.repository, &self.password_reset_config, &self.password_reset_mailer, input).await
    }

    async fn reset_password(&self, input: PasswordResetConfirm) -> AppResult<()> {
        let input = sanitize_password_reset_confirm(input);
        validate_password_reset_confirm(&input)?;
        let password_hash = self.password_hasher.hash(&input.password)?;
        reset_password(&self.repository, &input.token, &password_hash).await?;
        Ok(())
    }

    async fn authenticated_user(&self, id: UserId) -> AppResult<User> {
        if let Some(system_user) = system_user_by_id(&self.system_users, &id) {
            return Ok(system_user.user);
        }
        self.repository.find_by_id(id).await?.ok_or(AppError::Unauthorized)
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        let settings = self.registration_policy.registration_settings().await?;
        self.create_unique_user(input, &settings).await
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        reject_system_user_id(&self.system_users, &id)?;
        let input = sanitize_replace_user(input);
        validate_replace_user(&input)?;
        let current = self.repository.find_auth_by_id(id.clone()).await?.ok_or(AppError::NotFound)?;
        self.ensure_unique_user(&input.username, &input.email, Some(id.clone())).await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository
            .replace(id, self.replace_user_record(input)?.with_current_password_hash(current.password_hash))
            .await
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        reject_system_user_id(&self.system_users, &id)?;
        self.repository.delete(id).await
    }

    async fn list_users(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        validate_page(page)?;
        match self.system_users.system_user() {
            Some(system_user) => list_with_system_user(&self.repository, page, filters, system_user.user).await,
            None => self.repository.list(page, filters).await,
        }
    }

    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>> {
        self.wallets.wallet_summaries(user_ids).await
    }

    async fn identity_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, Vec<UserIdentitySummary>>> {
        let identities = self.repository.list_identities_by_user_ids(user_ids).await?;
        Ok(identities
            .into_iter()
            .map(|(user_id, items)| (user_id, items.into_iter().map(UserIdentitySummary::from).collect()))
            .collect())
    }

    async fn profile(&self, id: UserId) -> AppResult<User> {
        self.authenticated_user(id).await
    }

    async fn identities(&self, id: UserId) -> AppResult<Vec<UserIdentitySummary>> {
        let user = self.authenticated_user(id).await?;
        Ok(identity_summaries(self.repository.list_identities_by_user_id(&user.id.0).await?))
    }

    async fn request_account_password_email_code(&self, id: UserId, input: AccountPasswordEmailCodePayload) -> AppResult<()> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        social_auth::request_purpose_email_code(
            &self.purpose_email_code_store,
            &self.registration_email_config,
            &self.registration_email_mailer,
            ACCOUNT_PASSWORD_EMAIL_PURPOSE,
            RegistrationEmailCodeRequest {
                email: user.email,
                lang: input.lang,
            },
        )
        .await
    }

    async fn account_email_code_available(&self) -> AppResult<bool> {
        Ok(self.registration_email_config.account_email_settings().await?.is_ready())
    }

    async fn verify_account_email(&self, id: UserId, input: AccountEmailVerifyPayload) -> AppResult<User> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        social_auth::verify_email_with_code(&self.repository, &self.purpose_email_code_store, user, &input.email_verification_code).await
    }

    async fn change_account_password(&self, id: UserId, input: AccountPasswordChangePayload) -> AppResult<User> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        if self.account_email_code_available().await? {
            let code = input.email_verification_code.as_deref().unwrap_or_default();
            return social_auth::change_password_with_email_code(
                &self.repository,
                &self.purpose_email_code_store,
                &self.password_hasher,
                user,
                code,
                &input.password,
            )
            .await;
        }
        let current_password = input.current_password.as_deref().unwrap_or_default();
        let user_auth = self.repository.find_auth_by_id(user.id.clone()).await?.ok_or(AppError::NotFound)?;
        if user_auth.password_hash.is_none() {
            return Err(AppError::InvalidInput(ACCOUNT_EMAIL_CODE_DISABLED_MESSAGE.into()));
        }
        social_auth::change_password_with_current_password(&self.repository, &self.password_hasher, user_auth, current_password, &input.password).await
    }

    async fn unlink_identity(&self, id: UserId, identity_id: String) -> AppResult<()> {
        let user = self.authenticated_user(id).await?;
        reject_system_user_self_service(&user)?;
        unlink_identity(&self.repository, &user, &identity_id).await
    }

    async fn admin_user(&self, id: UserId) -> AppResult<User> {
        self.repository.find_by_id(id).await?.ok_or(AppError::NotFound)
    }

    async fn admin_unlink_identity(&self, user_id: UserId, identity_id: String) -> AppResult<()> {
        let user = self.repository.find_by_id(user_id).await?.ok_or(AppError::NotFound)?;
        unlink_identity(&self.repository, &user, &identity_id).await
    }
}
