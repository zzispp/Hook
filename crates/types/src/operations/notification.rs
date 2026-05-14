use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NotificationItem {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub category: String,
    pub is_unread: bool,
    pub created_at: String,
    pub link_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NotificationListFilters {
    pub status: Option<String>,
}
