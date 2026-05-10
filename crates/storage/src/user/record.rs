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
    pub password_hash: String,
    #[sea_orm(unique)]
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub is_deleted: bool,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
    pub last_login_at: Option<TimeDateTimeWithTimeZone>,
    pub auth_source: String,
    pub email_verified: bool,
    pub rate_limit_rpm: Option<i64>,
    pub quota_mode: String,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type UserRecord = Model;

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: UserId(value.id),
            username: value.username,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            system: false,
            rate_limit_rpm: value.rate_limit_rpm,
            quota_mode: value.quota_mode,
            created_at: format_timestamp(value.created_at),
            last_login_at: value.last_login_at.map(format_timestamp),
        }
    }
}

impl UserRecord {
    pub fn into_auth(self) -> UserAuthRecord {
        let password_hash = self.password_hash.clone();
        UserAuthRecord {
            user: self.into(),
            password_hash,
        }
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
