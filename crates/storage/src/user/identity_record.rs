use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::user::{IdentityProvider, UserIdentity};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_identities")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub provider_subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata_json: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
    pub last_login_at: Option<TimeDateTimeWithTimeZone>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type UserIdentityRecord = Model;

impl UserIdentityRecord {
    pub fn into_domain(self) -> crate::StorageResult<UserIdentity> {
        Ok(UserIdentity {
            id: self.id,
            user_id: self.user_id,
            provider: IdentityProvider::try_from(self.provider.as_str()).map_err(crate::StorageError::Database)?,
            provider_subject: self.provider_subject,
            email: self.email,
            email_verified: self.email_verified,
            display_name: self.display_name,
            avatar_url: self.avatar_url,
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
            last_login_at: self.last_login_at.map(format_timestamp),
        })
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("user identity timestamp must format as RFC3339")
}
