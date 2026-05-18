use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::application::{
    AppError, AppResult, InitialGrantLedger, PasswordHasher, PasswordResetConfig, PasswordResetConfirm, PasswordResetMailer, PasswordResetRepository,
    PasswordResetRequest, RegistrationPolicy, RegistrationSettings, ReplaceUserRecord, SystemUserProvider, UserAuthRecord, UserRepository, UserUseCase,
    UserWalletCatalog,
};
use types::{
    pagination::{Page, PageRequest},
    user::{Credentials, NewUser, ReplaceUser, User, UserId, UserListFilters, UserWalletSummaryResponse},
};

use self::{
    password_reset::{request_password_reset, reset_password},
    registration::{reject_closed_registration, reject_disallowed_registration_email},
    system_user::{find_auth_by_identifier, list_with_system_user, reject_conflicting_system_user, reject_system_user_id, system_user_by_id},
    validation::{
        sanitize_credentials, sanitize_new_user, sanitize_password_reset_confirm, sanitize_password_reset_request, sanitize_replace_user, validate_credentials,
        validate_new_user, validate_page, validate_password_reset_confirm, validate_password_reset_request, validate_replace_user,
    },
};

mod defaults;
mod password_reset;
mod registration;
mod system_user;
mod validation;

pub use defaults::{AllowRegistrationPolicy, NoInitialGrantLedger, NoPasswordResetConfig, NoPasswordResetMailer, NoSystemUserProvider, NoUserWalletCatalog};

pub struct UserService<
    R,
    H,
    S = NoSystemUserProvider,
    P = AllowRegistrationPolicy,
    G = NoInitialGrantLedger,
    W = NoUserWalletCatalog,
    C = NoPasswordResetConfig,
    M = NoPasswordResetMailer,
> {
    repository: R,
    password_hasher: H,
    system_users: S,
    registration_policy: P,
    initial_grants: G,
    wallets: W,
    password_reset_config: C,
    password_reset_mailer: M,
}

impl<R, H> UserService<R, H, NoSystemUserProvider, AllowRegistrationPolicy, NoInitialGrantLedger, NoUserWalletCatalog>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub const fn new(repository: R, password_hasher: H) -> Self {
        Self {
            repository,
            password_hasher,
            system_users: NoSystemUserProvider,
            registration_policy: AllowRegistrationPolicy,
            initial_grants: NoInitialGrantLedger,
            wallets: NoUserWalletCatalog,
            password_reset_config: NoPasswordResetConfig,
            password_reset_mailer: NoPasswordResetMailer,
        }
    }
}

impl<R, H, S> UserService<R, H, S, AllowRegistrationPolicy, NoInitialGrantLedger, NoUserWalletCatalog>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
{
    pub const fn with_system_user(repository: R, password_hasher: H, system_users: S) -> Self {
        Self {
            repository,
            password_hasher,
            system_users,
            registration_policy: AllowRegistrationPolicy,
            initial_grants: NoInitialGrantLedger,
            wallets: NoUserWalletCatalog,
            password_reset_config: NoPasswordResetConfig,
            password_reset_mailer: NoPasswordResetMailer,
        }
    }
}

impl<R, H, S, P, G, W> UserService<R, H, S, P, G, W>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
{
    pub const fn with_system_user_and_registration(
        repository: R,
        password_hasher: H,
        system_users: S,
        registration_policy: P,
        initial_grants: G,
        wallets: W,
    ) -> Self {
        Self {
            repository,
            password_hasher,
            system_users,
            registration_policy,
            initial_grants,
            wallets,
            password_reset_config: NoPasswordResetConfig,
            password_reset_mailer: NoPasswordResetMailer,
        }
    }
}

impl<R, H, S, P, G, W, C, M> UserService<R, H, S, P, G, W, C, M>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
{
    pub fn with_password_reset<NC, NM>(self, password_reset_config: NC, password_reset_mailer: NM) -> UserService<R, H, S, P, G, W, NC, NM> {
        UserService {
            repository: self.repository,
            password_hasher: self.password_hasher,
            system_users: self.system_users,
            registration_policy: self.registration_policy,
            initial_grants: self.initial_grants,
            wallets: self.wallets,
            password_reset_config,
            password_reset_mailer,
        }
    }

    async fn create_unique_user(&self, input: NewUser, settings: &RegistrationSettings) -> AppResult<User> {
        let input = sanitize_new_user(input);
        validate_new_user(&input)?;
        reject_disallowed_registration_email(settings, &input.email)?;
        self.create_valid_user(input).await
    }

    async fn create_valid_user(&self, input: NewUser) -> AppResult<User> {
        self.ensure_unique_user(&input.username, &input.email, None).await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, username: &str, email: &str, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(username).await? {
            reject_conflicting_user(found.user.id, current_id.as_ref(), "username")?;
        }

        if let Some(found) = self.repository.find_by_email(email).await? {
            reject_conflicting_user(found.id, current_id.as_ref(), "email")?;
        }

        Ok(())
    }

    fn ensure_unique_system_user(&self, username: &str, email: &str) -> AppResult<()> {
        reject_conflicting_system_user(&self.system_users, username, email)
    }

    fn new_user_record(&self, input: NewUser) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: Some(self.password_hasher.hash(&input.password)?),
            email: input.email,
            role: input.role,
            is_active: input.is_active,
            allowed_model_ids: input.allowed_model_ids,
            allowed_provider_ids: input.allowed_provider_ids,
            rate_limit_rpm: input.rate_limit_rpm,
            quota_mode: input.quota_mode,
        })
    }

    fn replace_user_record(&self, input: ReplaceUser) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: self.optional_password_hash(input.password)?,
            email: input.email,
            role: input.role,
            is_active: input.is_active,
            allowed_model_ids: input.allowed_model_ids,
            allowed_provider_ids: input.allowed_provider_ids,
            rate_limit_rpm: input.rate_limit_rpm,
            quota_mode: input.quota_mode,
        })
    }

    fn optional_password_hash(&self, password: Option<String>) -> AppResult<Option<String>> {
        password.map(|value| self.password_hasher.hash(&value)).transpose()
    }
}

#[async_trait]
impl<R, H, S, P, G, W, C, M> UserUseCase for UserService<R, H, S, P, G, W, C, M>
where
    R: UserRepository + PasswordResetRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
    C: PasswordResetConfig,
    M: PasswordResetMailer,
{
    async fn sign_up(&self, input: NewUser) -> AppResult<User> {
        let settings = self.registration_policy.registration_settings().await?;
        reject_closed_registration(&settings)?;
        let input = sanitize_new_user(input);
        validate_new_user(&input)?;
        reject_disallowed_registration_email(&settings, &input.email)?;
        let user = self.create_valid_user(input).await?;
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

fn reject_conflicting_user(id: UserId, current_id: Option<&UserId>, field: &str) -> AppResult<()> {
    if current_id == Some(&id) {
        return Ok(());
    }

    Err(AppError::Conflict(format!("{field} already exists")))
}

fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    if hasher.verify(password, &found.password_hash)? {
        return Ok(());
    }

    Err(AppError::InvalidCredentials)
}

#[cfg(test)]
mod registration_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
