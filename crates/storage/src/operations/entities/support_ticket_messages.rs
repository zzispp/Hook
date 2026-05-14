use sea_orm::entity::prelude::*;
use types::operations::SupportTicketMessage;

use crate::operations::time_format::format_timestamp;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "support_ticket_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub ticket_id: String,
    pub sender_user_id: String,
    pub sender_role: String,
    pub message_kind: String,
    pub body_markdown: String,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for SupportTicketMessage {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            ticket_id: value.ticket_id,
            sender_user_id: value.sender_user_id,
            sender_role: value.sender_role,
            message_kind: value.message_kind,
            body_markdown: value.body_markdown,
            created_at: format_timestamp(value.created_at),
        }
    }
}
