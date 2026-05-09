use async_trait::async_trait;
use types::api_token::{
    AdminApiTokenCreate, ApiTokenCreate, ApiTokenCreateResponse, ApiTokenListRequest, ApiTokenListResponse, ApiTokenResponse, ApiTokenSecretResponse,
    ApiTokenType, ApiTokenUpdate,
};

use crate::application::{
    ApiTokenCreateRecord, ApiTokenError, ApiTokenRepository, ApiTokenResult, ApiTokenUpdateRecord, BillingGroupCatalog, ModelAccessCatalog, SystemTokenPolicy,
    UserCatalog,
    token::{GeneratedToken, generate_token},
    validation::{
        ValidatedCreate, ValidatedUpdate, model_ids_for_update, sanitize_admin_create, sanitize_create, sanitize_update, validate_admin_create,
        validate_create, validate_list_request, validate_update,
    },
};

pub struct ApiTokenService<R, G, M, U, P> {
    repository: R,
    groups: G,
    models: M,
    users: U,
    system_policy: P,
}

impl<R, G, M, U, P> ApiTokenService<R, G, M, U, P>
where
    R: ApiTokenRepository,
    G: BillingGroupCatalog,
    M: ModelAccessCatalog,
    U: UserCatalog,
    P: SystemTokenPolicy,
{
    pub const fn new(repository: R, groups: G, models: M, users: U, system_policy: P) -> Self {
        Self {
            repository,
            groups,
            models,
            users,
            system_policy,
        }
    }
}

#[async_trait]
pub trait ApiTokenUseCase: Send + Sync + 'static {
    async fn create_token(&self, user_id: &str, input: ApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse>;
    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse>;
    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()>;
    async fn get_token(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenResponse>;
    async fn token_secret(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenSecretResponse>;
    async fn list_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse>;
    async fn create_admin_token(&self, actor_id: &str, input: AdminApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse>;
    async fn update_admin_token(&self, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse>;
    async fn delete_admin_token(&self, id: &str) -> ApiTokenResult<()>;
    async fn get_admin_token(&self, id: &str) -> ApiTokenResult<ApiTokenResponse>;
    async fn admin_token_secret(&self, id: &str) -> ApiTokenResult<ApiTokenSecretResponse>;
    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse>;
    async fn cleanup_expired_tokens(&self) -> ApiTokenResult<u64>;
}

#[async_trait]
impl<R, G, M, U, P> ApiTokenUseCase for ApiTokenService<R, G, M, U, P>
where
    R: ApiTokenRepository,
    G: BillingGroupCatalog,
    M: ModelAccessCatalog,
    U: UserCatalog,
    P: SystemTokenPolicy,
{
    async fn create_token(&self, user_id: &str, input: ApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
        let input = sanitize_create(input);
        let validated = validate_create(&input)?;
        self.ensure_create_policy(&validated.group_code, &validated.allowed_model_ids).await?;
        let generated = generate_token();
        let record = user_create_record(user_id, input, validated, &generated);
        self.create_response(record, generated).await
    }

    async fn update_token(&self, user_id: &str, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
        let current = self.repository.find_user_token(user_id, id).await?.ok_or(ApiTokenError::NotFound)?;
        let input = sanitize_update(input);
        let validated = validate_update(&current, &input)?;
        self.ensure_update_policy(&current, &input, &validated.allowed_model_ids).await?;
        let updated = self.repository.update_token(user_id, id, update_record(current, input, validated)).await?;
        Ok(updated.into())
    }

    async fn delete_token(&self, user_id: &str, id: &str) -> ApiTokenResult<()> {
        self.repository.delete_token(user_id, id).await
    }

    async fn get_token(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenResponse> {
        self.repository
            .find_user_token(user_id, id)
            .await?
            .map(Into::into)
            .ok_or(ApiTokenError::NotFound)
    }

    async fn token_secret(&self, user_id: &str, id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
        let token = self.repository.find_user_token(user_id, id).await?.ok_or(ApiTokenError::NotFound)?;
        Ok(ApiTokenSecretResponse { raw_token: token.token_value })
    }

    async fn list_tokens(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        validate_list_request(&request)?;
        self.user_token_list_response(user_id, request).await
    }

    async fn create_admin_token(&self, actor_id: &str, input: AdminApiTokenCreate) -> ApiTokenResult<ApiTokenCreateResponse> {
        let input = sanitize_admin_create(input);
        let validated = validate_admin_create(&input)?;
        let owner_id = admin_owner_id(actor_id, &input)?;
        self.ensure_create_policy(&validated.group_code, &validated.allowed_model_ids).await?;
        self.ensure_user_exists(&owner_id).await?;
        let generated = generate_token();
        let record = admin_create_record(owner_id, input, validated, &generated);
        self.create_response(record, generated).await
    }

    async fn update_admin_token(&self, id: &str, input: ApiTokenUpdate) -> ApiTokenResult<ApiTokenResponse> {
        let current = self.repository.find_token(id).await?.ok_or(ApiTokenError::NotFound)?;
        let input = sanitize_update(input);
        let validated = validate_update(&current, &input)?;
        self.ensure_update_policy(&current, &input, &validated.allowed_model_ids).await?;
        let updated = self.repository.update_any_token(id, update_record(current, input, validated)).await?;
        Ok(updated.into())
    }

    async fn delete_admin_token(&self, id: &str) -> ApiTokenResult<()> {
        self.repository.delete_any_token(id).await
    }

    async fn get_admin_token(&self, id: &str) -> ApiTokenResult<ApiTokenResponse> {
        self.repository.find_token(id).await?.map(Into::into).ok_or(ApiTokenError::NotFound)
    }

    async fn admin_token_secret(&self, id: &str) -> ApiTokenResult<ApiTokenSecretResponse> {
        let token = self.repository.find_token(id).await?.ok_or(ApiTokenError::NotFound)?;
        Ok(ApiTokenSecretResponse { raw_token: token.token_value })
    }

    async fn list_admin_tokens(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        validate_list_request(&request)?;
        self.admin_token_list_response(request).await
    }

    async fn cleanup_expired_tokens(&self) -> ApiTokenResult<u64> {
        if !self.system_policy.auto_delete_expired_tokens().await? {
            return Ok(0);
        }
        self.repository.delete_expired_tokens().await
    }
}

impl<R, G, M, U, P> ApiTokenService<R, G, M, U, P>
where
    R: ApiTokenRepository,
    G: BillingGroupCatalog,
    M: ModelAccessCatalog,
    U: UserCatalog,
    P: SystemTokenPolicy,
{
    async fn user_token_list_response(&self, user_id: &str, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        let mut response = self.repository.list_user_tokens(user_id, request).await?;
        apply_default_rate_limit(&mut response, self.system_policy.default_rate_limit_rpm().await?);
        Ok(response)
    }

    async fn admin_token_list_response(&self, request: ApiTokenListRequest) -> ApiTokenResult<ApiTokenListResponse> {
        let mut response = self.repository.list_admin_tokens(request).await?;
        apply_default_rate_limit(&mut response, self.system_policy.default_rate_limit_rpm().await?);
        Ok(response)
    }

    async fn create_response(&self, record: ApiTokenCreateRecord, generated: GeneratedToken) -> ApiTokenResult<ApiTokenCreateResponse> {
        let token = self.repository.create_token(record).await?;
        Ok(ApiTokenCreateResponse {
            token: token.into(),
            raw_token: generated.value,
        })
    }

    async fn ensure_create_policy(&self, group_code: &str, model_ids: &[String]) -> ApiTokenResult<()> {
        let group = self.active_group(group_code).await?;
        self.ensure_models_exist(model_ids).await?;
        ensure_group_allows_models(&group, model_ids)
    }

    async fn ensure_update_policy(&self, current: &types::api_token::ApiToken, input: &ApiTokenUpdate, model_ids: &[String]) -> ApiTokenResult<()> {
        let group_code = input.group_code.as_deref().unwrap_or(&current.group_code);
        let group = self.active_group(group_code).await?;
        self.ensure_models_exist(model_ids).await?;
        ensure_group_allows_models(&group, model_ids)
    }

    async fn active_group(&self, code: &str) -> ApiTokenResult<types::group::BillingGroupResponse> {
        self.groups
            .active_group(code)
            .await?
            .ok_or_else(|| ApiTokenError::InvalidInput(format!("active billing group does not exist: {code}")))
    }

    async fn ensure_models_exist(&self, ids: &[String]) -> ApiTokenResult<()> {
        for id in ids {
            if !self.models.model_exists(id).await? {
                return Err(ApiTokenError::InvalidInput(format!("global model does not exist: {id}")));
            }
        }
        Ok(())
    }

    async fn ensure_user_exists(&self, id: &str) -> ApiTokenResult<()> {
        if self.users.user_exists(id).await? {
            return Ok(());
        }
        Err(ApiTokenError::InvalidInput(format!("user does not exist: {id}")))
    }
}

fn apply_default_rate_limit(response: &mut ApiTokenListResponse, default_rate: i64) {
    for token in &mut response.tokens {
        if token.rate_limit_rpm == Some(0) {
            token.rate_limit_rpm = Some(default_rate);
        }
    }
}

fn ensure_group_allows_models(group: &types::group::BillingGroupResponse, model_ids: &[String]) -> ApiTokenResult<()> {
    if group.allowed_model_ids.is_empty() {
        return Ok(());
    }
    for id in model_ids {
        if !group.allowed_model_ids.iter().any(|allowed| allowed == id) {
            return Err(ApiTokenError::InvalidInput(format!(
                "model is not allowed by billing group {}: {id}",
                group.code
            )));
        }
    }
    Ok(())
}

fn user_create_record(user_id: &str, input: ApiTokenCreate, validated: ValidatedCreate, generated: &GeneratedToken) -> ApiTokenCreateRecord {
    ApiTokenCreateRecord {
        user_id: user_id.into(),
        token_type: ApiTokenType::User,
        name: input.name,
        token_value: generated.value.clone(),
        token_hash: generated.hash.clone(),
        token_prefix: generated.prefix.clone(),
        group_code: validated.group_code,
        expires_at: validated.expires_at,
        model_access_mode: validated.model_access_mode,
        allowed_model_ids: validated.allowed_model_ids,
        rate_limit_rpm: Some(input.rate_limit_rpm.unwrap_or(0)),
        quota_limit: input.quota_limit,
    }
}

fn admin_create_record(owner_id: String, input: AdminApiTokenCreate, validated: ValidatedCreate, generated: &GeneratedToken) -> ApiTokenCreateRecord {
    ApiTokenCreateRecord {
        user_id: owner_id,
        token_type: input.token_type,
        name: input.name,
        token_value: generated.value.clone(),
        token_hash: generated.hash.clone(),
        token_prefix: generated.prefix.clone(),
        group_code: validated.group_code,
        expires_at: validated.expires_at,
        model_access_mode: validated.model_access_mode,
        allowed_model_ids: validated.allowed_model_ids,
        rate_limit_rpm: Some(input.rate_limit_rpm.unwrap_or(0)),
        quota_limit: input.quota_limit,
    }
}

fn update_record(current: types::api_token::ApiToken, input: ApiTokenUpdate, validated: ValidatedUpdate) -> ApiTokenUpdateRecord {
    let allowed_model_ids = model_ids_for_update(&current, &validated, &input);
    ApiTokenUpdateRecord {
        name: input.name,
        group_code: input.group_code,
        expires_at: validated.expires_at,
        model_access_mode: input.model_access_mode,
        allowed_model_ids,
        rate_limit_rpm: input.rate_limit_rpm,
        quota_limit: input.quota_limit,
        is_active: input.is_active,
    }
}

fn admin_owner_id(actor_id: &str, input: &AdminApiTokenCreate) -> ApiTokenResult<String> {
    match input.token_type {
        ApiTokenType::Independent => Ok(actor_id.to_owned()),
        ApiTokenType::User => input
            .user_id
            .clone()
            .ok_or_else(|| ApiTokenError::InvalidInput("user_id is required for user token".into())),
    }
}
