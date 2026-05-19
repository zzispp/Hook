use crate::application::{
    AppResult, InitialGrantLedger, PasswordHasher, RegistrationPolicy, RegistrationSettings, ReplaceUserRecord, SystemUserProvider, UserRepository,
    UserWalletCatalog,
};
use registration::reject_disallowed_registration_email;
use system_user::reject_conflicting_system_user;
use types::user::{NewUser, ReplaceUser, User, UserId};
use validation::{sanitize_new_user, validate_new_user};

mod defaults;
mod password_reset;
mod registration;
mod system_user;
mod use_cases;
mod validation;

pub use defaults::{
    AllowRegistrationPolicy, NoInitialGrantLedger, NoPasswordResetConfig, NoPasswordResetMailer, NoRegistrationEmailConfig, NoRegistrationEmailMailer,
    NoSystemUserProvider, NoUserWalletCatalog,
};

pub struct UserService<
    R,
    H,
    S = NoSystemUserProvider,
    P = AllowRegistrationPolicy,
    G = NoInitialGrantLedger,
    W = NoUserWalletCatalog,
    C = NoPasswordResetConfig,
    M = NoPasswordResetMailer,
    E = NoRegistrationEmailConfig,
    N = NoRegistrationEmailMailer,
> {
    repository: R,
    password_hasher: H,
    system_users: S,
    registration_policy: P,
    initial_grants: G,
    wallets: W,
    password_reset_config: C,
    password_reset_mailer: M,
    registration_email_config: E,
    registration_email_mailer: N,
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
            registration_email_config: NoRegistrationEmailConfig,
            registration_email_mailer: NoRegistrationEmailMailer,
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
            registration_email_config: NoRegistrationEmailConfig,
            registration_email_mailer: NoRegistrationEmailMailer,
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
            registration_email_config: NoRegistrationEmailConfig,
            registration_email_mailer: NoRegistrationEmailMailer,
        }
    }
}

impl<R, H, S, P, G, W, C, M, E, N> UserService<R, H, S, P, G, W, C, M, E, N>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
{
    pub fn with_password_reset<NC, NM>(self, password_reset_config: NC, password_reset_mailer: NM) -> UserService<R, H, S, P, G, W, NC, NM, E, N> {
        UserService {
            repository: self.repository,
            password_hasher: self.password_hasher,
            system_users: self.system_users,
            registration_policy: self.registration_policy,
            initial_grants: self.initial_grants,
            wallets: self.wallets,
            password_reset_config,
            password_reset_mailer,
            registration_email_config: self.registration_email_config,
            registration_email_mailer: self.registration_email_mailer,
        }
    }

    pub fn with_registration_email<NE, NN>(
        self,
        registration_email_config: NE,
        registration_email_mailer: NN,
    ) -> UserService<R, H, S, P, G, W, C, M, NE, NN> {
        UserService {
            repository: self.repository,
            password_hasher: self.password_hasher,
            system_users: self.system_users,
            registration_policy: self.registration_policy,
            initial_grants: self.initial_grants,
            wallets: self.wallets,
            password_reset_config: self.password_reset_config,
            password_reset_mailer: self.password_reset_mailer,
            registration_email_config,
            registration_email_mailer,
        }
    }

    async fn create_unique_user(&self, input: NewUser, settings: &RegistrationSettings) -> AppResult<User> {
        let input = sanitize_new_user(input);
        validate_new_user(&input)?;
        reject_disallowed_registration_email(settings, &input.email)?;
        self.create_valid_user(input, false).await
    }

    async fn create_valid_user(&self, input: NewUser, email_verified: bool) -> AppResult<User> {
        self.ensure_unique_user(&input.username, &input.email, None).await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input, Some(email_verified))?).await
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

    fn new_user_record(&self, input: NewUser, email_verified: Option<bool>) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: Some(self.password_hasher.hash(&input.password)?),
            email: input.email,
            email_verified,
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
            email_verified: None,
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

fn reject_conflicting_user(id: UserId, current_id: Option<&UserId>, field: &str) -> AppResult<()> {
    if current_id == Some(&id) {
        return Ok(());
    }

    Err(crate::application::AppError::Conflict(format!("{field} already exists")))
}

#[cfg(test)]
mod registration_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
