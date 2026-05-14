use sea_orm::entity::prelude::*;
use types::operations::SupportTicketEmailEvent;

use crate::operations::time_format::format_timestamp;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "support_ticket_email_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub ticket_id: String,
    pub message_id: Option<String>,
    pub recipient_email: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for SupportTicketEmailEvent {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            ticket_id: value.ticket_id,
            message_id: value.message_id,
            recipient_email: value.recipient_email,
            subject: value.subject,
            status: value.status,
            error_message: value.error_message,
            created_at: format_timestamp(value.created_at),
        }
    }
}
