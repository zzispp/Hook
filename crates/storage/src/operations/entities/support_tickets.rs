use sea_orm::entity::prelude::*;
use types::operations::SupportTicket;

use crate::operations::time_format::format_timestamp;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "support_tickets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub subject: String,
    pub contact_email: String,
    pub status: String,
    pub priority: String,
    pub last_message_at: TimeDateTimeWithTimeZone,
    pub last_message_sender_role: String,
    pub last_user_activity_at: Option<TimeDateTimeWithTimeZone>,
    pub last_admin_activity_at: Option<TimeDateTimeWithTimeZone>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for SupportTicket {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            subject: value.subject,
            contact_email: value.contact_email,
            status: value.status,
            priority: value.priority,
            last_message_at: format_timestamp(value.last_message_at),
            last_message_sender_role: value.last_message_sender_role,
            last_user_activity_at: value.last_user_activity_at.map(format_timestamp),
            last_admin_activity_at: value.last_admin_activity_at.map(format_timestamp),
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}
