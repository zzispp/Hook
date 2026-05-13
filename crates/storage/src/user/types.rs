use types::user::User;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserRecordInput {
    pub username: String,
    pub password_hash: Option<String>,
    pub email: String,
    pub role: String,
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
