use sea_orm::entity::prelude::*;
use types::operations::Announcement;

use crate::operations::time_format::format_timestamp;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "announcements")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: String,
    pub content_markdown: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub priority: i64,
    pub enabled: bool,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Announcement {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            content_markdown: value.content_markdown,
            announcement_type: value.announcement_type,
            pinned: value.pinned,
            priority: value.priority,
            enabled: value.enabled,
            created_by: value.created_by,
            updated_by: value.updated_by,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}
