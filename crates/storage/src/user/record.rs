use constants::auth::DEFAULT_AUTH_SOURCE;
use types::user::{User, UserId};

use super::UserAuthRecord;

#[derive(Clone, Debug, toasty::Model)]
#[table = "users"]
pub struct UserRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[unique]
    #[column(type = varchar(100))]
    pub username: String,
    #[column(type = varchar(255))]
    pub password_hash: String,
    #[unique]
    #[column(type = varchar(255))]
    pub email: String,
    #[column(type = varchar(100))]
    pub role: String,
    pub is_active: bool,
    pub is_deleted: bool,
    #[auto]
    #[column(type = timestamp(6))]
    pub created_at: jiff::Timestamp,
    #[auto]
    #[column(type = timestamp(6))]
    pub updated_at: jiff::Timestamp,
    #[column(type = timestamp(6))]
    pub last_login_at: Option<jiff::Timestamp>,
    #[column(type = varchar(50))]
    pub auth_source: String,
    pub email_verified: bool,
}

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
}

impl UserRecord {
    pub fn local_auth_source() -> String {
        DEFAULT_AUTH_SOURCE.into()
    }
}
