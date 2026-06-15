use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "routing_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub profile_id: String,
    pub profile_version: String,
    pub profile_config: String,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
