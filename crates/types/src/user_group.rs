use serde::{Deserialize, Serialize};

use crate::pagination::{Page, PageRequest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserGroup {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct UserGroupFilters {
    pub search: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct UserGroupListRequest {
    pub page: PageRequest,
    pub filters: UserGroupFilters,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct UserGroupListQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct UserGroupCreate {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct UserGroupUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserGroupResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub type UserGroupPageResponse = Page<UserGroupResponse>;

impl From<UserGroupListQuery> for UserGroupListRequest {
    fn from(value: UserGroupListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            filters: UserGroupFilters {
                search: value.search,
                is_active: value.is_active,
            },
        }
    }
}

impl From<UserGroup> for UserGroupResponse {
    fn from(value: UserGroup) -> Self {
        Self {
            id: value.id,
            code: value.code,
            name: value.name,
            description: value.description,
            is_active: value.is_active,
            is_system: value.is_system,
            sort_order: value.sort_order,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
