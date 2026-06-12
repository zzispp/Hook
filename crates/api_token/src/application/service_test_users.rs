use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use types::api_token::ApiTokenOwnerResponse;

use super::{ApiTokenResult, UserCatalog};

pub(super) const ADMIN_USER_ID: &str = "admin-user-1";
pub(super) const SYSTEM_ACTOR_ID: &str = "00000000-0000-7000-8000-000000000000";
pub(super) const USER_ID: &str = "user-1";

const ADMIN_ROLE: &str = "admin";

#[derive(Clone)]
pub(super) struct ExistingUsers {
    users: Arc<Vec<ExistingUser>>,
}

#[derive(Clone)]
struct ExistingUser {
    id: String,
    role: String,
}

impl ExistingUsers {
    pub(super) fn empty() -> Self {
        Self { users: Arc::new(Vec::new()) }
    }

    pub(super) fn with<const N: usize>(ids: [&str; N]) -> Self {
        Self {
            users: Arc::new(ids.into_iter().map(regular_user).collect()),
        }
    }

    pub(super) fn with_admin<const N: usize>(ids: [&str; N]) -> Self {
        Self {
            users: Arc::new(ids.into_iter().map(admin_user).collect()),
        }
    }
}

#[async_trait]
impl UserCatalog for ExistingUsers {
    async fn user_exists(&self, id: &str) -> ApiTokenResult<bool> {
        Ok(self.users.iter().any(|existing| existing.id == id))
    }

    async fn user_group_codes(&self, id: &str) -> ApiTokenResult<Option<Vec<String>>> {
        if id == SYSTEM_ACTOR_ID || self.users.iter().any(|existing| existing.id == id) {
            return Ok(Some(vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()]));
        }
        Ok(None)
    }

    async fn user_role(&self, id: &str) -> ApiTokenResult<Option<String>> {
        if id == SYSTEM_ACTOR_ID {
            return Ok(Some(ADMIN_ROLE.into()));
        }
        Ok(self.users.iter().find(|existing| existing.id == id).map(|user| user.role.clone()))
    }

    async fn owners_by_id(&self, ids: &[String]) -> ApiTokenResult<BTreeMap<String, ApiTokenOwnerResponse>> {
        Ok(ids
            .iter()
            .filter(|id| self.users.iter().any(|existing| existing.id == **id))
            .map(|id| owner_response(id))
            .collect())
    }
}

fn regular_user(id: &str) -> ExistingUser {
    ExistingUser {
        id: id.into(),
        role: constants::auth::DEFAULT_USER_ROLE.into(),
    }
}

fn admin_user(id: &str) -> ExistingUser {
    ExistingUser {
        id: id.into(),
        role: ADMIN_ROLE.into(),
    }
}

fn owner_response(id: &str) -> (String, ApiTokenOwnerResponse) {
    (
        id.into(),
        ApiTokenOwnerResponse {
            username: id.into(),
            email: format!("{id}@example.test"),
            group_codes: vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()],
        },
    )
}
