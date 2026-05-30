use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use types::{
    api_token::{
        AdminApiTokenCreate, ApiTokenCreate, ApiTokenCreateResponse, ApiTokenListRequest, ApiTokenListResponse, ApiTokenResponse, ApiTokenSecretResponse,
        ApiTokenUpdate,
    },
    response::ApiResponse,
};

use crate::api::{ApiTokenApiError, ApiTokenApiState};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, ApiTokenApiError>;

pub async fn list_tokens(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ApiTokenListRequest>,
) -> ApiResult<ApiJson<ApiTokenListResponse>> {
    Ok(ok(state.tokens.list_tokens(&current_user.id, query).await?))
}

pub async fn get_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.get_token(&current_user.id, &id).await?))
}

pub async fn token_secret(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<ApiTokenSecretResponse>> {
    Ok(ok(state.tokens.token_secret(&current_user.id, &id).await?))
}

pub async fn create_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ApiTokenCreate>,
) -> ApiResult<ApiJson<ApiTokenCreateResponse>> {
    Ok(ok(state.tokens.create_token(&current_user.id, payload).await?))
}

pub async fn update_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<ApiTokenUpdate>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.update_token(&current_user.id, &id, payload).await?))
}

pub async fn delete_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<()>> {
    state.tokens.delete_token(&current_user.id, &id).await?;
    Ok(ok(()))
}

pub async fn list_admin_tokens(State(state): State<ApiTokenApiState>, Query(query): Query<ApiTokenListRequest>) -> ApiResult<ApiJson<ApiTokenListResponse>> {
    Ok(ok(state.tokens.list_admin_tokens(query).await?))
}

pub async fn get_admin_token(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.get_admin_token(&id).await?))
}

pub async fn admin_token_secret(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ApiTokenSecretResponse>> {
    Ok(ok(state.tokens.admin_token_secret(&id).await?))
}

pub async fn create_admin_token(
    State(state): State<ApiTokenApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AdminApiTokenCreate>,
) -> ApiResult<ApiJson<ApiTokenCreateResponse>> {
    Ok(ok(state.tokens.create_admin_token(&current_user.id, payload).await?))
}

pub async fn update_admin_token(
    State(state): State<ApiTokenApiState>,
    Path(id): Path<String>,
    Json(payload): Json<ApiTokenUpdate>,
) -> ApiResult<ApiJson<ApiTokenResponse>> {
    Ok(ok(state.tokens.update_admin_token(&id, payload).await?))
}

pub async fn delete_admin_token(State(state): State<ApiTokenApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.tokens.delete_admin_token(&id).await?;
    Ok(ok(()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use axum::{
        Extension,
        extract::{Query, State},
    };
    use rbac::api::CurrentUser;
    use types::api_token::{
        AdminApiTokenCreate, ApiTokenCreate, ApiTokenCreateResponse, ApiTokenListRequest, ApiTokenListResponse, ApiTokenResponse, ApiTokenSecretResponse,
        ApiTokenUpdate,
    };

    use super::{list_admin_tokens, list_tokens};
    use crate::{
        api::ApiTokenApiState,
        application::{ApiTokenResult, ApiTokenUseCase},
    };

    const USER_ID: &str = "user-1";
    const EXPECTED_LIST_CALLS: u64 = 1;
    #[tokio::test]
    async fn list_tokens_does_not_trigger_expired_cleanup() {
        let tokens = Arc::new(RecordingTokens::default());
        let state = ApiTokenApiState::new(tokens.clone());

        let _ = list_tokens(State(state), Extension(current_user()), Query(ApiTokenListRequest::default()))
            .await
            .unwrap();

        assert_eq!(tokens.user_list_calls(), EXPECTED_LIST_CALLS);
    }

    #[tokio::test]
    async fn list_admin_tokens_does_not_trigger_expired_cleanup() {
        let tokens = Arc::new(RecordingTokens::default());
        let state = ApiTokenApiState::new(tokens.clone());

        let _ = list_admin_tokens(State(state), Query(ApiTokenListRequest::default())).await.unwrap();

        assert_eq!(tokens.admin_list_calls(), EXPECTED_LIST_CALLS);
    }

    #[derive(Default)]
    struct RecordingTokens {
        user_list_calls: Mutex<u64>,
        admin_list_calls: Mutex<u64>,
    }

    impl RecordingTokens {
        fn user_list_calls(&self) -> u64 {
            *self.user_list_calls.lock().unwrap()
        }

        fn admin_list_calls(&self) -> u64 {
            *self.admin_list_calls.lock().unwrap()
        }
    }

    #[async_trait]
    impl ApiTokenUseCase for RecordingTokens {
        async fn create_token(&self, _user_id: &str, _input: ApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn update_token(&self, _user_id: &str, _id: &str, _input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn delete_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<()> {
            unimplemented!("not needed for list handler tests")
        }

        async fn get_token(&self, _user_id: &str, _id: &str) -> ApiTokenResult<ApiTokenResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn token_secret(&self, _user_id: &str, _id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn list_tokens(&self, user_id: &str, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
            assert_eq!(user_id, USER_ID);
            *self.user_list_calls.lock().unwrap() += 1;
            Ok(empty_list())
        }

        async fn create_admin_token(&self, _actor_id: &str, _input: AdminApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn update_admin_token(&self, _id: &str, _input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn delete_admin_token(&self, _id: &str) -> ApiTokenResult<()> {
            unimplemented!("not needed for list handler tests")
        }

        async fn get_admin_token(&self, _id: &str) -> ApiTokenResult<ApiTokenResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn admin_token_secret(&self, _id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
            unimplemented!("not needed for list handler tests")
        }

        async fn list_admin_tokens(&self, _request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
            *self.admin_list_calls.lock().unwrap() += 1;
            Ok(empty_list())
        }
    }

    fn empty_list() -> ApiTokenListResponse {
        ApiTokenListResponse { tokens: Vec::new(), total: 0 }
    }

    fn current_user() -> CurrentUser {
        CurrentUser {
            id: USER_ID.into(),
            username: "alice".into(),
            role: "user".into(),
            group_codes: vec!["default".into()],
            system: false,
        }
    }
}
