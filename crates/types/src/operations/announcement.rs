use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub content_markdown: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub enabled: bool,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct AnnouncementInput {
    pub title: String,
    pub content_markdown: String,
    pub announcement_type: String,
    pub pinned: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct AnnouncementPatch {
    pub title: Option<String>,
    pub content_markdown: Option<String>,
    pub announcement_type: Option<String>,
    pub pinned: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnnouncementListFilters {
    pub search: Option<String>,
    pub announcement_type: Option<String>,
    pub enabled: Option<bool>,
}
