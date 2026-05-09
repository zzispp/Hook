use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "menu_api_permissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_item_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub api_permission_id: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
