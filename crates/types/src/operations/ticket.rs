use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicket {
    pub id: String,
    pub user_id: String,
    pub subject: String,
    pub contact_email: String,
    pub status: String,
    pub priority: String,
    pub last_message_at: String,
    pub last_message_sender_role: String,
    pub last_user_activity_at: Option<String>,
    pub last_admin_activity_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicketMessage {
    pub id: String,
    pub ticket_id: String,
    pub sender_user_id: String,
    pub sender_role: String,
    pub message_kind: String,
    pub body_markdown: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicketEmailEvent {
    pub id: String,
    pub ticket_id: String,
    pub message_id: Option<String>,
    pub recipient_email: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicketCreatePayload {
    pub subject: String,
    pub body_markdown: String,
    #[serde(default)]
    pub contact_email: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicketMessagePayload {
    pub body_markdown: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct SupportTicketPatch {
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SupportTicketMutationResponse {
    pub ticket: SupportTicket,
    pub message: Option<SupportTicketMessage>,
    pub email_delivery: SupportTicketEmailDelivery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SupportTicketEmailDelivery {
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SupportTicketDetail {
    pub ticket: SupportTicket,
    pub messages: Vec<SupportTicketMessage>,
    pub email_events: Vec<SupportTicketEmailEvent>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportTicketListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportTicketCreateInput {
    pub user_id: String,
    pub subject: String,
    pub body_markdown: String,
    pub contact_email: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportTicketMessageInput {
    pub ticket_id: String,
    pub sender_user_id: String,
    pub sender_role: String,
    pub body_markdown: String,
}
