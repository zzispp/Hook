use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest},
    user::{
        AuthConfigResponse, Credentials, NewUser, PasswordResetConfirm, PasswordResetRequest, RegistrationEmailCodeRequest, ReplaceUser, SignUpUser, User,
        UserId, UserListFilters, UserWalletSummaryResponse,
    },
};

use crate::application::{
    AppError, AppResult, InitialGrantLedger, PasswordHasher, PasswordResetConfig, PasswordResetMailer, PasswordResetRepository, RegistrationEmailCodeStore,
    RegistrationEmailConfig, RegistrationEmailMailer, RegistrationPolicy, SystemUserProvider, UserAuthRecord, UserRepository, UserUseCase, UserWalletCatalog,
};

use super::{
    UserService,
    password_reset::{request_password_reset, reset_password},
    registration::{reject_closed_registration, reject_disallowed_registration_email, request_registration_email_code, verify_registration_email_code},
    system_user::{find_auth_by_identifier, list_with_system_user, reject_system_user_id, system_user_by_id},
    validation::{
        sanitize_credentials, sanitize_password_reset_confirm, sanitize_password_reset_request, sanitize_registration_email_code_request,
        sanitize_replace_user, sanitize_sign_up_user, validate_credentials, validate_new_user, validate_page, validate_password_reset_confirm,
        validate_password_reset_request, validate_registration_email_code_request, validate_replace_user,
    },
};

#[async_trait]
impl<R, H, S, P, G, W, C, M, E, N, K> UserUseCase for UserService<R, H, S, P, G, W, C, M, E, N, K>
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
        let input = sanitize_sign_up_user(input);
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
}

async fn grant_initial_balance<G>(ledger: &G, user: &User, amount: Decimal) -> AppResult<()>
where
    G: InitialGrantLedger,
{
    if amount <= Decimal::ZERO {
        return Ok(());
    }
    ledger.grant_initial_balance(&user.id.0, amount).await
}

fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    if hasher.verify(password, &found.password_hash)? {
        return Ok(());
    }
    Err(AppError::InvalidCredentials)
}
