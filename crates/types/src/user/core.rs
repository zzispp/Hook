use serde::{Deserialize, Serialize};

pub const USER_QUOTA_MODE_WALLET: &str = "wallet";
pub const USER_QUOTA_MODE_UNLIMITED: &str = "unlimited";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub auth_source: String,
    pub email_verified: bool,
    pub system: bool,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUser {
    pub username: String,
    pub password: Option<String>,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub identifier: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetRequest {
    pub email: String,
    pub lang: String,
    pub reset_origin: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserListFilters {
    pub search: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

pub fn default_user_created_at() -> String {
    "1970-01-01T00:00:00Z".into()
}
