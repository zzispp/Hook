use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::user::AffiliateRelationChangeRecord;

use super::record::Entity as Users;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "affiliate_relation_changes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub old_referrer_user_id: Option<String>,
    pub new_referrer_user_id: Option<String>,
    pub operator_user_id: Option<String>,
    pub reason: String,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Users", from = "Column::UserId", to = "super::record::Column::Id")]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<Users> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl From<Model> for AffiliateRelationChangeRecord {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            old_referrer_user_id: value.old_referrer_user_id,
            new_referrer_user_id: value.new_referrer_user_id,
            operator_user_id: value.operator_user_id,
            reason: value.reason,
            created_at: format_timestamp(value.created_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("affiliate relation change timestamp must format as RFC3339")
}
