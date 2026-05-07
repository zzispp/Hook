use types::user::{User, UserId};

use super::UserAuthRecord;

#[derive(Clone, Debug, toasty::Model)]
pub struct UserRecord {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub username: String,
    pub password_hash: String,
    #[unique]
    pub email: String,
    pub role: String,
    pub status: String,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: UserId(value.id),
            username: value.username,
            email: value.email,
            role: value.role,
            status: value.status,
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
