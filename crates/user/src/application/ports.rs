use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    system_setting::EmailSuffixMode,
    user::{Credentials, NewUser, ReplaceUser, User, UserId, UserListFilters, UserWalletSummaryResponse},
};

use super::AppResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUserRecord {
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub role: String,
    pub is_active: bool,
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
    pub default_user_grant: Decimal,
    pub email_suffix_mode: EmailSuffixMode,
    pub email_suffixes: String,
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
    async fn sign_up(&self, input: NewUser) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<User>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
    async fn create_user(&self, input: NewUser) -> AppResult<User>;
    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User>;
    async fn delete_user(&self, id: UserId) -> AppResult<()>;
    async fn list_users(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>>;
    async fn wallet_summaries(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>>;
}
