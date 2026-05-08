use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "menu_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub section_id: String,
    pub parent_id: Option<String>,
    #[sea_orm(unique)]
    pub code: String,
    pub title: String,
    pub route_path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
