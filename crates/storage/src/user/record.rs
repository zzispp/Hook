use constants::auth::DEFAULT_AUTH_SOURCE;
use sea_orm::entity::prelude::*;
use types::user::{User, UserId};

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
}
