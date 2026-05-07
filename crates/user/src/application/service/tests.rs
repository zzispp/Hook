use constants::pagination::MAX_PAGE_SIZE;
use types::user::{Credentials, PageRequest, UserId};

use crate::{
    application::{AppError, UserService, UserUseCase},
    test_support::{MemoryUserRepository, TestPasswordHasher, new_user, replace_user, stored_user},
};

#[tokio::test]
async fn sign_up_hashes_password_and_persists_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.sign_up(new_user("alice")).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].password_hash, "hashed:secret");
}

#[tokio::test]
async fn sign_in_rejects_invalid_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn sign_in_accepts_email_identifier() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .sign_in(Credentials {
            identifier: "alice@example.com".into(),
            password: "secret".into(),
        })
        .await
        .unwrap();

    assert_eq!(user.username, "alice");
}

#[tokio::test]
async fn authenticated_user_returns_user_from_token_subject() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.authenticated_user(UserId(1)).await.unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn authenticated_user_rejects_unknown_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.authenticated_user(UserId(1)).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_user(new_user("alice")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn replace_user_allows_same_user_identity() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.replace_user(UserId(1), replace_user("alice", "active")).await.unwrap();

    assert_eq!(user.status, "active");
    assert_eq!(repository.replaced_records()[0].1.password_hash, "hashed:secret");
}

#[tokio::test]
async fn list_users_rejects_zero_page() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(PageRequest { page: 0, page_size: 10 }).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[tokio::test]
async fn list_users_rejects_page_size_above_maximum() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .list_users(PageRequest {
            page: 1,
            page_size: MAX_PAGE_SIZE + 1,
        })
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}
