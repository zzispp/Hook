use serde::{Deserialize, Serialize};

use super::{Credentials, NewUser, Page, PageRequest, ReplaceUser, User};

#[derive(Debug, Deserialize)]
pub struct UserPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct UsersPageResponse {
    pub items: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<UserPayload> for NewUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            status: value.status,
        }
    }
}

impl From<UserPayload> for ReplaceUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            status: value.status,
        }
    }
}

impl From<SignInPayload> for Credentials {
    fn from(value: SignInPayload) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}

impl From<ListUsersQuery> for PageRequest {
    fn from(value: ListUsersQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id.0,
            username: value.username,
            email: value.email,
            role: value.role,
            status: value.status,
        }
    }
}

impl From<Page<User>> for UsersPageResponse {
    fn from(value: Page<User>) -> Self {
        Self {
            items: value.items.into_iter().map(UserResponse::from).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}
