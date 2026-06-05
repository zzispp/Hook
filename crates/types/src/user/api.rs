use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::pagination::{Page, PageRequest};

use super::{Credentials, NewUser, PasswordResetConfirm, PasswordResetRequest, ReplaceUser, User, UserIdentitySummary, UserListFilters};

#[derive(Debug, Deserialize)]
pub struct UserPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    #[serde(default)]
    pub group_codes: Option<Vec<String>>,
    pub is_active: bool,
    #[serde(default)]
    pub allowed_model_ids: Vec<String>,
    #[serde(default)]
    pub allowed_provider_ids: Vec<String>,
    #[serde(default)]
    pub rate_limit_rpm: Option<i64>,
    #[serde(default)]
    pub quota_mode: Option<String>,
    #[serde(default)]
    pub referrer_aff_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignUpPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    #[serde(default)]
    pub email_verification_code: Option<String>,
    #[serde(default)]
    pub captcha_token: Option<String>,
    #[serde(default)]
    pub aff_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationEmailCodePayload {
    pub email: String,
    pub lang: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInPayload {
    pub identifier: String,
    pub password: String,
    #[serde(default)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestPayload {
    pub email: String,
    pub lang: String,
    pub reset_origin: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmPayload {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub group_code: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub group_codes: Vec<String>,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub auth_source: String,
    pub email_verified: bool,
    pub password_set: bool,
    pub system: bool,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
    pub affiliate_code: String,
    pub referred_by_user_id: Option<String>,
    pub referred_at: Option<String>,
    pub created_at: String,
    pub last_login_at: Option<String>,
    pub wallet: Option<UserWalletSummaryResponse>,
    pub identities: Vec<UserIdentitySummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UserWalletSummaryResponse {
    pub id: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub available_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_consumed: Decimal,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct UsersPageResponse {
    pub items: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<UserPayload> for NewUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            group_codes: value.group_codes,
            is_active: value.is_active,
            allowed_model_ids: value.allowed_model_ids,
            allowed_provider_ids: value.allowed_provider_ids,
            rate_limit_rpm: value.rate_limit_rpm,
            quota_mode: value.quota_mode.unwrap_or_else(|| super::USER_QUOTA_MODE_WALLET.into()),
            referrer_aff_code: value.referrer_aff_code,
        }
    }
}

impl From<UserPayload> for ReplaceUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: Some(value.password),
            email: value.email,
            role: value.role,
            group_codes: value.group_codes.unwrap_or_default(),
            is_active: value.is_active,
            allowed_model_ids: value.allowed_model_ids,
            allowed_provider_ids: value.allowed_provider_ids,
            rate_limit_rpm: value.rate_limit_rpm,
            quota_mode: value.quota_mode.unwrap_or_else(|| super::USER_QUOTA_MODE_WALLET.into()),
        }
    }
}

impl From<SignInPayload> for Credentials {
    fn from(value: SignInPayload) -> Self {
        Self {
            identifier: value.identifier,
            password: value.password,
        }
    }
}

impl From<PasswordResetRequestPayload> for PasswordResetRequest {
    fn from(value: PasswordResetRequestPayload) -> Self {
        Self {
            email: value.email,
            lang: value.lang,
            reset_origin: value.reset_origin,
        }
    }
}

impl From<PasswordResetConfirmPayload> for PasswordResetConfirm {
    fn from(value: PasswordResetConfirmPayload) -> Self {
        Self {
            token: value.token,
            password: value.password,
        }
    }
}

impl From<RegistrationEmailCodePayload> for super::RegistrationEmailCodeRequest {
    fn from(value: RegistrationEmailCodePayload) -> Self {
        Self {
            email: value.email,
            lang: value.lang,
        }
    }
}

impl From<ListUsersQuery> for PageRequest {
    fn from(value: ListUsersQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<ListUsersQuery> for UserListFilters {
    fn from(value: ListUsersQuery) -> Self {
        Self {
            search: value.search,
            role: value.role,
            group_code: value.group_code,
            is_active: value.is_active,
        }
    }
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id.0,
            username: value.username,
            email: value.email,
            role: value.role,
            group_codes: value.group_codes,
            is_active: value.is_active,
            allowed_model_ids: value.allowed_model_ids,
            allowed_provider_ids: value.allowed_provider_ids,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            password_set: value.password_set,
            system: value.system,
            rate_limit_rpm: value.rate_limit_rpm,
            quota_mode: value.quota_mode,
            affiliate_code: value.affiliate_code,
            referred_by_user_id: value.referred_by_user_id.map(|id| id.0),
            referred_at: value.referred_at,
            created_at: value.created_at,
            last_login_at: value.last_login_at,
            wallet: None,
            identities: Vec::new(),
        }
    }
}

impl UserResponse {
    pub fn with_wallet(mut self, wallet: Option<UserWalletSummaryResponse>) -> Self {
        self.wallet = wallet;
        self
    }

    pub fn with_identities(mut self, identities: Vec<UserIdentitySummary>) -> Self {
        self.identities = identities;
        self
    }
}

impl From<Page<User>> for UsersPageResponse {
    fn from(value: Page<User>) -> Self {
        Self {
            items: value.items.into_iter().map(UserResponse::from).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}
