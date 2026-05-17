use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dimension_collectors")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub api_format: String,
    pub task_type: String,
    pub dimension_name: String,
    pub source_type: String,
    pub source_path: Option<String>,
    pub value_type: String,
    pub transform_expression: Option<String>,
    pub default_value: Option<String>,
    pub priority: i32,
    pub is_enabled: bool,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
