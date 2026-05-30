use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    system_setting::{EmailSuffixMode, SmtpEncryption},
    user::{
        AccountPasswordChangePayload, AccountPasswordEmailCodePayload, AuthConfigResponse, Credentials, IdentityProvider, NewUser, PasswordResetConfirm,
        PasswordResetRequest, RegistrationEmailCodeRequest, ReplaceUser, SignUpUser, User, UserId, UserIdentity, UserIdentityInput, UserIdentitySummary,
        UserListFilters, UserWalletSummaryResponse,
    },
    user_group::{UserGroupCreate, UserGroupListRequest, UserGroupPageResponse, UserGroupResponse, UserGroupUpdate},
};

use super::AppResult;
use super::{OAuthSignInResult, WalletChallenge, WalletNonceInput, WalletSignInInput, WalletSignInResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUserRecord {
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub email_verified: Option<bool>,
    pub role: String,
    pub group_codes: Vec<String>,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

impl ReplaceUserRecord {
    pub fn with_current_password_hash(self, current_password_hash: Option<String>) -> Self {
        Self {
            password_hash: self.password_hash.or(current_password_hash),
            ..self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAuthRecord {
    pub user: User,
    pub password_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetRecord {
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemUserRecord {
    pub user: User,
    pub password_hash: String,
}

pub trait SystemUserProvider: Send + Sync + 'static {
    fn system_user(&self) -> Option<SystemUserRecord>;
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User>;
    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User>;
    async fn delete(&self, id: UserId) -> AppResult<()>;
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>>;
    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>>;
    async fn record_login(&self, id: UserId) -> AppResult<()>;
    async fn list(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
    async fn list_slice(&self, request: PageSliceRequest, filters: UserListFilters) -> AppResult<Page<User>>;
    async fn create_identity(&self, input: UserIdentityInput) -> AppResult<UserIdentity>;
    async fn find_identity(&self, provider: IdentityProvider, subject: &str) -> AppResult<Option<UserIdentity>>;
    async fn list_identities_by_user_id(&self, user_id: &str) -> AppResult<Vec<UserIdentity>>;
    async fn list_identities_by_user_ids(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, Vec<UserIdentity>>>;
    async fn touch_identity_login(&self, identity_id: &str) -> AppResult<()>;
    async fn delete_identity(&self, identity_id: &str) -> AppResult<()>;
}

pub trait PasswordHasher: Send + Sync + 'static {
    fn hash(&self, password: &str) -> AppResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait RegistrationPolicy: Send + Sync + 'static {
    async fn registration_settings(&self) -> AppResult<RegistrationSettings>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrationSettings {
    pub allow_registration: bool,
    pub registration_email_verification_enabled: bool,
    pub default_user_grant: Decimal,
    pub default_user_group_code: String,
    pub email_suffix_mode: EmailSuffixMode,
    pub email_suffixes: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmailSettings {
    pub site_name: String,
    pub feature_enabled: bool,
    pub email_config_enabled: bool,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password_set: bool,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_encryption: SmtpEncryption,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetTemplate {
    pub subject: String,
    pub html: String,
}

#[async_trait]
pub trait PasswordResetRepository: Send + Sync + 'static {
    async fn create_password_reset_token(&self, record: PasswordResetRecord) -> AppResult<()>;
    async fn consume_password_reset_token(&self, token_hash: &str, password_hash: &str, now: time::OffsetDateTime) -> AppResult<Option<User>>;
}

#[async_trait]
pub trait PasswordResetConfig: Send + Sync + 'static {
    async fn password_reset_settings(&self) -> AppResult<EmailSettings>;
    async fn password_reset_template(&self, lang: &str) -> AppResult<PasswordResetTemplate>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetEmail {
    pub recipient_email: String,
    pub subject: String,
    pub html: String,
    pub settings: EmailSettings,
}

#[async_trait]
pub trait PasswordResetMailer: Send + Sync + 'static {
    async fn send_password_reset(&self, email: PasswordResetEmail) -> AppResult<()>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrationEmailTemplate {
    pub subject: String,
    pub html: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrationEmail {
    pub recipient_email: String,
    pub subject: String,
    pub html: String,
    pub settings: EmailSettings,
}

#[async_trait]
pub trait RegistrationEmailConfig: Send + Sync + 'static {
    async fn auth_config(&self) -> AppResult<AuthConfigResponse>;
    async fn registration_email_settings(&self) -> AppResult<EmailSettings>;
    async fn registration_email_template(&self, lang: &str) -> AppResult<RegistrationEmailTemplate>;
}

#[async_trait]
pub trait RegistrationEmailMailer: Send + Sync + 'static {
    async fn send_registration_email(&self, email: RegistrationEmail) -> AppResult<()>;
}

#[async_trait]
pub trait RegistrationEmailCodeStore: Send + Sync + 'static {
    async fn active_registration_email_code(&self, email: &str) -> AppResult<Option<String>>;
    async fn save_registration_email_code(&self, email: &str, code: &str, ttl_seconds: u64) -> AppResult<()>;
    async fn begin_registration_email_code_cooldown(&self, email: &str, ttl_seconds: u64) -> AppResult<bool>;
    async fn consume_registration_email_code(&self, email: &str, code: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait InitialGrantLedger: Send + Sync + 'static {
    async fn grant_initial_balance(&self, user_id: &str, amount: Decimal) -> AppResult<()>;
}

#[async_trait]
pub trait UserWalletCatalog: Send + Sync + 'static {
    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserGroupCreateRecord {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserGroupUpdateRecord {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

#[async_trait]
pub trait UserGroupRepository: Send + Sync + 'static {
    async fn create_group(&self, input: UserGroupCreateRecord) -> AppResult<UserGroupResponse>;
    async fn update_group(&self, code: &str, input: UserGroupUpdateRecord) -> AppResult<UserGroupResponse>;
    async fn delete_group(&self, code: &str) -> AppResult<()>;
    async fn find_group(&self, code: &str) -> AppResult<Option<UserGroupResponse>>;
    async fn list_groups(&self, request: UserGroupListRequest) -> AppResult<UserGroupPageResponse>;
    async fn group_has_users(&self, code: &str) -> AppResult<bool>;
    async fn list_group_users(&self, request: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
}

#[async_trait]
pub trait UserGroupBillingCatalog: Send + Sync + 'static {
    async fn user_group_has_billing_groups(&self, code: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait UserGroupSettingCatalog: Send + Sync + 'static {
    async fn default_user_group_code(&self) -> AppResult<String>;
}

#[async_trait]
pub trait UserGroupUseCase: Send + Sync + 'static {
    async fn create_user_group(&self, input: UserGroupCreate) -> AppResult<UserGroupResponse>;
    async fn update_user_group(&self, code: &str, input: UserGroupUpdate) -> AppResult<UserGroupResponse>;
    async fn delete_user_group(&self, code: &str) -> AppResult<()>;
    async fn get_user_group(&self, code: &str) -> AppResult<UserGroupResponse>;
    async fn list_user_groups(&self, request: UserGroupListRequest) -> AppResult<UserGroupPageResponse>;
    async fn list_user_group_members(&self, code: &str, request: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
}

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn auth_config(&self) -> AppResult<AuthConfigResponse>;
    async fn request_registration_email_code(&self, input: RegistrationEmailCodeRequest) -> AppResult<()>;
    async fn sign_up(&self, input: SignUpUser) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<User>;
    async fn oauth_start(&self, provider: IdentityProvider) -> AppResult<String>;
    async fn oauth_callback(&self, provider: IdentityProvider, code: String, state: String) -> AppResult<OAuthSignInResult>;
    async fn bind_oauth_existing(&self, provider: IdentityProvider, ticket: String) -> AppResult<User>;
    async fn wallet_nonce(&self, input: WalletNonceInput) -> AppResult<WalletChallenge>;
    async fn wallet_sign_in(&self, input: WalletSignInInput) -> AppResult<WalletSignInResult>;
    async fn request_wallet_email_code(&self, ticket: String, email: String, lang: String) -> AppResult<()>;
    async fn complete_wallet(&self, ticket: String, email: String, code: String) -> AppResult<User>;
    async fn request_password_reset(&self, input: PasswordResetRequest) -> AppResult<()>;
    async fn reset_password(&self, input: PasswordResetConfirm) -> AppResult<()>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
    async fn create_user(&self, input: NewUser) -> AppResult<User>;
    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User>;
    async fn delete_user(&self, id: UserId) -> AppResult<()>;
    async fn list_users(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>>;
    async fn identity_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, Vec<UserIdentitySummary>>>;
    async fn profile(&self, id: UserId) -> AppResult<User>;
    async fn identities(&self, id: UserId) -> AppResult<Vec<UserIdentitySummary>>;
    async fn request_account_password_email_code(&self, id: UserId, input: AccountPasswordEmailCodePayload) -> AppResult<()>;
    async fn change_account_password(&self, id: UserId, input: AccountPasswordChangePayload) -> AppResult<User>;
    async fn unlink_identity(&self, id: UserId, identity_id: String) -> AppResult<()>;
    async fn admin_user(&self, id: UserId) -> AppResult<User>;
    async fn admin_unlink_identity(&self, user_id: UserId, identity_id: String) -> AppResult<()>;
}
