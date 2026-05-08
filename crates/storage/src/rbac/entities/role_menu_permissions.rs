use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "role_menu_permissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_code: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub menu_item_id: String,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
