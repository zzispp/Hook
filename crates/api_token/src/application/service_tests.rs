use types::api_token::ApiTokenType;

use super::{
    ApiTokenUseCase,
    service_test_support::{ExistingUsers, MemoryTokenRepository, SYSTEM_ACTOR_ID, USER_ID, admin_create, record_owner, service, token_with_type, user_create},
};

#[tokio::test]
async fn admin_independent_token_uses_actor_as_owner() {
    let repository = MemoryTokenRepository::default();
    let service = service(repository.clone(), ExistingUsers::empty());

    let created = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::Independent, None))
        .await
        .unwrap();

    assert_eq!(created.token.user_id, Some(SYSTEM_ACTOR_ID.into()));
    assert_eq!(created.token.token_type, ApiTokenType::Independent);
    assert_eq!(
        repository.created_records(),
        vec![record_owner(Some(SYSTEM_ACTOR_ID), ApiTokenType::Independent)]
    );
}

#[tokio::test]
async fn admin_independent_token_ignores_payload_user_id() {
    let repository = MemoryTokenRepository::default();
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let created = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::Independent, Some(USER_ID)))
        .await
        .unwrap();

    assert_eq!(created.token.user_id, Some(SYSTEM_ACTOR_ID.into()));
    assert_eq!(
        repository.created_records(),
        vec![record_owner(Some(SYSTEM_ACTOR_ID), ApiTokenType::Independent)]
    );
}

#[tokio::test]
async fn admin_user_token_requires_existing_user() {
    let repository = MemoryTokenRepository::default();
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let created = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, Some(USER_ID)))
        .await
        .unwrap();

    assert_eq!(created.token.user_id, Some(USER_ID.into()));
    assert_eq!(created.token.token_type, ApiTokenType::User);
    assert_eq!(repository.created_records(), vec![record_owner(Some(USER_ID), ApiTokenType::User)]);
}

#[tokio::test]
async fn admin_user_token_rejects_missing_user_id() {
    let service = service(MemoryTokenRepository::default(), ExistingUsers::with([USER_ID]));

    let result = service.create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, None)).await;

    assert!(result.is_err_and(|error| error.to_string().contains("user_id is required")));
}

#[tokio::test]
async fn admin_user_token_rejects_unknown_user() {
    let service = service(MemoryTokenRepository::default(), ExistingUsers::empty());

    let result = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, Some(USER_ID)))
        .await;

    assert!(result.is_err_and(|error| error.to_string().contains("user does not exist")));
}

#[tokio::test]
async fn user_token_create_rejects_owner_limit() {
    let repository = MemoryTokenRepository::with_owner_token_count(5);
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let result = service.create_token(USER_ID, user_create()).await;

    assert_eq!(result.unwrap_err().to_string(), "api token conflict: token quantity limit reached");
    assert!(repository.created_records().is_empty());
}

#[tokio::test]
async fn admin_user_token_create_rejects_owner_limit() {
    let repository = MemoryTokenRepository::with_owner_token_count(5);
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let result = service
        .create_admin_token(SYSTEM_ACTOR_ID, admin_create(ApiTokenType::User, Some(USER_ID)))
        .await;

    assert_eq!(result.unwrap_err().to_string(), "api token conflict: token quantity limit reached");
    assert!(repository.created_records().is_empty());
}

#[tokio::test]
async fn delete_admin_independent_token_rejects_model_status_binding() {
    let token = token_with_type("token-1", ApiTokenType::Independent);
    let repository = MemoryTokenRepository::with_token(token, true);
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    let result = service.delete_admin_token("token-1").await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "api token conflict: independent token is bound to model status checks"
    );
    assert!(repository.deleted_ids().is_empty());
}

#[tokio::test]
async fn delete_admin_user_token_ignores_model_status_binding_check() {
    let token = token_with_type("token-1", ApiTokenType::User);
    let repository = MemoryTokenRepository::with_token(token, true);
    let service = service(repository.clone(), ExistingUsers::with([USER_ID]));

    service.delete_admin_token("token-1").await.unwrap();

    assert_eq!(repository.deleted_ids(), vec!["token-1".to_owned()]);
}
