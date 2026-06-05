use crate::application::{
    AppResult, AuthProviderConfig, AuthTicketStore, InitialGrantLedger, OAuthClient, PasswordHasher, PurposeEmailCodeStore, RegistrationPolicy,
    RegistrationSettings, ReplaceUserRecord, SystemUserProvider, UserRepository, UserWalletCatalog,
};
use registration::reject_disallowed_registration_email;
use system_user::reject_conflicting_system_user;
use types::user::{NewUser, ReplaceUser, User, UserId};
use validation::{sanitize_new_user, validate_new_user};

mod admin_affiliate;
mod affiliate;
mod defaults;
mod password_reset;
mod registration;
mod social_auth;
mod system_user;
mod use_cases;
mod user_group;
mod validation;

pub use defaults::{
    AllowRegistrationPolicy, NoAuthProviderConfig, NoAuthTicketStore, NoInitialGrantLedger, NoOAuthClient, NoPasswordResetConfig, NoPasswordResetMailer,
    NoPurposeEmailCodeStore, NoRegistrationEmailCodeStore, NoRegistrationEmailConfig, NoRegistrationEmailMailer, NoSystemUserProvider, NoUserWalletCatalog,
};
pub use user_group::UserGroupService;

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
    K = NoRegistrationEmailCodeStore,
    A = NoAuthProviderConfig,
    O = NoOAuthClient,
    T = NoAuthTicketStore,
    Y = NoPurposeEmailCodeStore,
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
    registration_email_code_store: K,
    auth_provider_config: A,
    oauth_client: O,
    auth_ticket_store: T,
    purpose_email_code_store: Y,
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
            registration_email_code_store: NoRegistrationEmailCodeStore,
            auth_provider_config: NoAuthProviderConfig,
            oauth_client: NoOAuthClient,
            auth_ticket_store: NoAuthTicketStore,
            purpose_email_code_store: NoPurposeEmailCodeStore,
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
            registration_email_code_store: NoRegistrationEmailCodeStore,
            auth_provider_config: NoAuthProviderConfig,
            oauth_client: NoOAuthClient,
            auth_ticket_store: NoAuthTicketStore,
            purpose_email_code_store: NoPurposeEmailCodeStore,
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
            registration_email_code_store: NoRegistrationEmailCodeStore,
            auth_provider_config: NoAuthProviderConfig,
            oauth_client: NoOAuthClient,
            auth_ticket_store: NoAuthTicketStore,
            purpose_email_code_store: NoPurposeEmailCodeStore,
        }
    }
}

impl<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y> UserService<R, H, S, P, G, W, C, M, E, N, K, A, O, T, Y>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
    P: RegistrationPolicy,
    G: InitialGrantLedger,
    W: UserWalletCatalog,
{
    #[allow(clippy::type_complexity)]
    pub fn with_password_reset<NC, NM>(
        self,
        password_reset_config: NC,
        password_reset_mailer: NM,
    ) -> UserService<R, H, S, P, G, W, NC, NM, E, N, K, A, O, T, Y> {
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
            registration_email_code_store: self.registration_email_code_store,
            auth_provider_config: self.auth_provider_config,
            oauth_client: self.oauth_client,
            auth_ticket_store: self.auth_ticket_store,
            purpose_email_code_store: self.purpose_email_code_store,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn with_registration_email<NE, NN, NK>(
        self,
        registration_email_config: NE,
        registration_email_mailer: NN,
        registration_email_code_store: NK,
    ) -> UserService<R, H, S, P, G, W, C, M, NE, NN, NK, A, O, T, Y> {
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
            registration_email_code_store,
            auth_provider_config: self.auth_provider_config,
            oauth_client: self.oauth_client,
            auth_ticket_store: self.auth_ticket_store,
            purpose_email_code_store: self.purpose_email_code_store,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn with_social_auth<NA, NO, NT, NY>(
        self,
        auth_provider_config: NA,
        oauth_client: NO,
        auth_ticket_store: NT,
        purpose_email_code_store: NY,
    ) -> UserService<R, H, S, P, G, W, C, M, E, N, K, NA, NO, NT, NY>
    where
        NA: AuthProviderConfig,
        NO: OAuthClient,
        NT: AuthTicketStore,
        NY: PurposeEmailCodeStore,
    {
        UserService {
            repository: self.repository,
            password_hasher: self.password_hasher,
            system_users: self.system_users,
            registration_policy: self.registration_policy,
            initial_grants: self.initial_grants,
            wallets: self.wallets,
            password_reset_config: self.password_reset_config,
            password_reset_mailer: self.password_reset_mailer,
            registration_email_config: self.registration_email_config,
            registration_email_mailer: self.registration_email_mailer,
            registration_email_code_store: self.registration_email_code_store,
            auth_provider_config,
            oauth_client,
            auth_ticket_store,
            purpose_email_code_store,
        }
    }

    async fn create_unique_user(&self, input: NewUser, settings: &RegistrationSettings) -> AppResult<User> {
        let input = sanitize_new_user(with_default_user_group(input, &settings.default_user_group_code));
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
            group_codes: input.group_codes.unwrap_or_default(),
            is_active: input.is_active,
            allowed_model_ids: input.allowed_model_ids,
            allowed_provider_ids: input.allowed_provider_ids,
            rate_limit_rpm: input.rate_limit_rpm,
            quota_mode: input.quota_mode,
            referrer_aff_code: input.referrer_aff_code,
        })
    }

    fn replace_user_record(&self, input: ReplaceUser) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: self.optional_password_hash(input.password)?,
            email: input.email,
            email_verified: None,
            role: input.role,
            group_codes: input.group_codes,
            is_active: input.is_active,
            allowed_model_ids: input.allowed_model_ids,
            allowed_provider_ids: input.allowed_provider_ids,
            rate_limit_rpm: input.rate_limit_rpm,
            quota_mode: input.quota_mode,
            referrer_aff_code: None,
        })
    }

    fn optional_password_hash(&self, password: Option<String>) -> AppResult<Option<String>> {
        password.map(|value| self.password_hasher.hash(&value)).transpose()
    }
}

fn with_default_user_group(mut input: NewUser, default_user_group_code: &str) -> NewUser {
    if input.group_codes.is_none() {
        input.group_codes = Some(vec![default_user_group_code.to_owned()]);
    }
    input
}

fn provider_user_record(input: NewUser, email_verified: Option<bool>) -> ReplaceUserRecord {
    ReplaceUserRecord {
        username: input.username,
        password_hash: None,
        email: input.email,
        email_verified,
        role: input.role,
        group_codes: input.group_codes.unwrap_or_default(),
        is_active: input.is_active,
        allowed_model_ids: input.allowed_model_ids,
        allowed_provider_ids: input.allowed_provider_ids,
        rate_limit_rpm: input.rate_limit_rpm,
        quota_mode: input.quota_mode,
        referrer_aff_code: input.referrer_aff_code,
    }
}

fn reject_conflicting_user(id: UserId, current_id: Option<&UserId>, field: &str) -> AppResult<()> {
    if current_id == Some(&id) {
        return Ok(());
    }

    Err(crate::application::AppError::Conflict(format!("{field} already exists")))
}

#[cfg(test)]
mod registration_email_test_support;
#[cfg(test)]
mod registration_email_tests;
#[cfg(test)]
mod registration_tests;
#[cfg(test)]
mod social_auth_email_test_support;
#[cfg(test)]
mod social_auth_test_support;
#[cfg(test)]
mod social_auth_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
