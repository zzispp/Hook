use constants::auth::DEFAULT_AUTH_SOURCE;
use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::user::{USER_QUOTA_MODE_WALLET, User, UserId};

use super::UserAuthRecord;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: Option<String>,
    #[sea_orm(unique)]
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub is_deleted: bool,
    pub allowed_model_ids: String,
    pub allowed_provider_ids: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
    pub last_login_at: Option<TimeDateTimeWithTimeZone>,
    pub auth_source: String,
    pub email_verified: bool,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
    #[sea_orm(unique)]
    pub affiliate_code: String,
    pub referred_by_user_id: Option<String>,
    pub referred_at: Option<TimeDateTimeWithTimeZone>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type UserRecord = Model;

impl UserRecord {
    pub fn into_domain(self, group_codes: Vec<String>) -> crate::StorageResult<User> {
        let allowed_model_ids = serde_json::from_str(&self.allowed_model_ids)?;
        let allowed_provider_ids = serde_json::from_str(&self.allowed_provider_ids)?;
        Ok(User {
            id: UserId(self.id),
            username: self.username,
            email: self.email,
            role: self.role,
            group_codes,
            is_active: self.is_active,
            allowed_model_ids,
            allowed_provider_ids,
            auth_source: self.auth_source,
            email_verified: self.email_verified,
            password_set: self.password_hash.is_some(),
            system: false,
            rate_limit_rpm: self.rate_limit_rpm,
            quota_mode: self.quota_mode,
            affiliate_code: self.affiliate_code,
            referred_by_user_id: self.referred_by_user_id.map(UserId),
            referred_at: self.referred_at.map(format_timestamp),
            created_at: format_timestamp(self.created_at),
            last_login_at: self.last_login_at.map(format_timestamp),
        })
    }

    pub fn into_auth(self, group_codes: Vec<String>) -> crate::StorageResult<UserAuthRecord> {
        let password_hash = self.password_hash.clone();
        Ok(UserAuthRecord {
            user: self.into_domain(group_codes)?,
            password_hash,
        })
    }

    pub fn local_auth_source() -> String {
        DEFAULT_AUTH_SOURCE.into()
    }

    pub fn default_quota_mode() -> String {
        USER_QUOTA_MODE_WALLET.into()
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("user timestamp must format as RFC3339")
}
