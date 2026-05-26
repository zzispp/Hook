use types::user::User;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserRecordInput {
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub email_verified: Option<bool>,
    pub role: String,
    pub group_code: String,
    pub is_active: bool,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAuthRecord {
    pub user: User,
    pub password_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserGroupRecordInput {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserGroupRecordPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetTokenRecordInput {
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordResetTokenRecord {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: time::OffsetDateTime,
    pub consumed_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}
