use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnouncementRecordInput {
    pub title: String,
    pub content_markdown: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub priority: i64,
    pub enabled: bool,
    pub operator_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnnouncementRecordPatch {
    pub title: Option<String>,
    pub content_markdown: Option<String>,
    pub announcement_type: Option<String>,
    pub pinned: Option<bool>,
    pub priority: Option<i64>,
    pub enabled: Option<bool>,
    pub operator_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketRecordInput {
    pub user_id: String,
    pub subject: String,
    pub contact_email: String,
    pub body_markdown: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketMessageRecordInput {
    pub ticket_id: String,
    pub sender_user_id: String,
    pub sender_role: String,
    pub body_markdown: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TicketRecordPatch {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub operator_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmailEventRecordInput {
    pub ticket_id: String,
    pub message_id: Option<String>,
    pub recipient_email: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationSourceRecord {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub category: String,
    pub event_at: OffsetDateTime,
    pub link_path: String,
}
