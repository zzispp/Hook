use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    system_setting::{EmailSuffixMode, SmtpEncryption},
    user::{
        AuthConfigResponse, Credentials, NewUser, PasswordResetConfirm, PasswordResetRequest, RegistrationEmailCodeRequest, ReplaceUser, SignUpUser, User,
        UserId, UserListFilters, UserWalletSummaryResponse,
    },
};

use super::AppResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUserRecord {
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub email_verified: Option<bool>,
    pub role: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

impl ReplaceUserRecord {
    pub fn with_current_password_hash(self, current_password_hash: String) -> Self {
        Self {
            password_hash: Some(self.password_hash.unwrap_or(current_password_hash)),
            ..self
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAuthRecord {
    pub user: User,
    pub password_hash: String,
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

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn auth_config(&self) -> AppResult<AuthConfigResponse>;
    async fn request_registration_email_code(&self, input: RegistrationEmailCodeRequest) -> AppResult<()>;
    async fn sign_up(&self, input: SignUpUser) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<User>;
    async fn request_password_reset(&self, input: PasswordResetRequest) -> AppResult<()>;
    async fn reset_password(&self, input: PasswordResetConfirm) -> AppResult<()>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
    async fn create_user(&self, input: NewUser) -> AppResult<User>;
    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User>;
    async fn delete_user(&self, id: UserId) -> AppResult<()>;
    async fn list_users(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>>;
}
