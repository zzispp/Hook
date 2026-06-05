use constants::pagination::MAX_PAGE_SIZE;
use types::{
    pagination::PageRequest,
    user::{AdminAffiliateRelationUpdateRequest, Credentials, NewUser, SignUpUser, default_user_created_at},
};

use crate::{
    application::{AdminAffiliateUseCase, AffiliateUseCase, AppError, UserRepository, UserService, UserUseCase},
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, affiliate_code, new_user, replace_user, stored_user, user_id},
};

#[tokio::test]
async fn sign_up_hashes_password_and_persists_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.sign_up(sign_up_user(new_user("alice"))).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].password_hash.as_deref(), Some(format!("hashed:{VALID_PASSWORD}").as_str()));
}

#[tokio::test]
async fn sign_up_generates_affiliate_code() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.sign_up(sign_up_user(new_user("alice"))).await.unwrap();

    assert_eq!(user.affiliate_code, affiliate_code(&user.id));
    assert_eq!(user.referred_by_user_id, None);
}

#[tokio::test]
async fn sign_up_binds_referrer_from_aff_code() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "referrer", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .sign_up(SignUpUser {
            user: new_user("alice"),
            email_verification_code: None,
            aff_code: Some(affiliate_code(&user_id(1))),
        })
        .await
        .unwrap();

    assert_eq!(user.referred_by_user_id, Some(user_id(1)));
    assert_eq!(user.referred_at, Some(default_user_created_at()));
}

#[tokio::test]
async fn sign_up_rejects_unknown_aff_code() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .sign_up(SignUpUser {
            user: new_user("alice"),
            email_verification_code: None,
            aff_code: Some("missing-aff".into()),
        })
        .await;

    assert!(matches!(result, Err(AppError::Conflict(message)) if message == "referrer affiliate code does not exist"));
    assert_eq!(repository.created_records().len(), 0);
}

#[tokio::test]
async fn create_user_binds_referrer_only_when_explicit() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "referrer", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .create_user(NewUser {
            referrer_aff_code: Some(affiliate_code(&user_id(1))),
            ..new_user("alice")
        })
        .await
        .unwrap();

    assert_eq!(user.referred_by_user_id, Some(user_id(1)));
}

#[tokio::test]
async fn create_user_defaults_to_no_referrer() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.create_user(new_user("alice")).await.unwrap();

    assert_eq!(user.referred_by_user_id, None);
}

#[tokio::test]
async fn affiliate_summary_returns_code_link_count_and_total() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "referrer", "hashed:secret123"),
        stored_user(2, "alice", "hashed:secret123").referred_by(user_id(1)),
    ]);
    let service = UserService::new(repository, TestPasswordHasher);

    let summary = service.affiliate_summary(user_id(1)).await.unwrap();

    assert_eq!(summary.affiliate_code, affiliate_code(&user_id(1)));
    assert_eq!(summary.affiliate_link, format!("/auth/sign-up?aff={}", affiliate_code(&user_id(1))));
    assert_eq!(summary.referred_user_count, 1);
    assert_eq!(summary.total_commission_amount, rust_decimal::Decimal::ZERO);
}

#[tokio::test]
async fn admin_affiliate_rebind_updates_referrer_with_reason() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "referrer", "hashed:secret123").regular_user(),
        stored_user(2, "alice", "hashed:secret123"),
    ]);
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    service
        .update_admin_affiliate_relation(
            &user_id(2).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: Some(affiliate_code(&user_id(1))),
                clear_referrer: false,
                reason: "support request".into(),
            },
            Some(user_id(1).0),
        )
        .await
        .unwrap();

    let user = repository.find_by_id(user_id(2)).await.unwrap().unwrap();
    assert_eq!(user.referred_by_user_id, Some(user_id(1)));
}

#[tokio::test]
async fn admin_affiliate_rebind_accepts_virtual_system_operator() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "referrer", "hashed:secret123").regular_user(),
        stored_user(2, "alice", "hashed:secret123"),
    ]);
    let service = UserService::new(repository, TestPasswordHasher);

    let change = service
        .update_admin_affiliate_relation(
            &user_id(2).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: Some(affiliate_code(&user_id(1))),
                clear_referrer: false,
                reason: "support request".into(),
            },
            None,
        )
        .await;

    assert_eq!(change.unwrap().operator_user_id, None);
}

#[tokio::test]
async fn admin_affiliate_rebind_keeps_database_operator_id() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "referrer", "hashed:secret123").regular_user(),
        stored_user(2, "alice", "hashed:secret123"),
    ]);
    let service = UserService::new(repository, TestPasswordHasher);

    let change = service
        .update_admin_affiliate_relation(
            &user_id(2).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: Some(affiliate_code(&user_id(1))),
                clear_referrer: false,
                reason: "support request".into(),
            },
            Some(user_id(1).0),
        )
        .await;

    assert_eq!(change.unwrap().operator_user_id, Some(user_id(1).0));
}

#[tokio::test]
async fn admin_affiliate_rebind_rejects_admin_referrer() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "admin_referrer", "hashed:secret123"),
        stored_user(2, "alice", "hashed:secret123"),
    ]);
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .update_admin_affiliate_relation(
            &user_id(2).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: Some(affiliate_code(&user_id(1))),
                clear_referrer: false,
                reason: "support request".into(),
            },
            Some(user_id(1).0),
        )
        .await;

    assert!(matches!(result, Err(AppError::Conflict(message)) if message == "only regular users can be referrers"));
}

#[tokio::test]
async fn admin_affiliate_update_requires_reason() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .update_admin_affiliate_relation(
            &user_id(1).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: None,
                clear_referrer: true,
                reason: "  ".into(),
            },
            Some(user_id(1).0),
        )
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "reason is required"));
}

#[tokio::test]
async fn admin_affiliate_rebind_requires_affiliate_code() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .update_admin_affiliate_relation(
            &user_id(1).0,
            AdminAffiliateRelationUpdateRequest {
                referrer_aff_code: None,
                clear_referrer: false,
                reason: "support request".into(),
            },
            Some(user_id(1).0),
        )
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "referrer_aff_code is required"));
}

#[tokio::test]
async fn sign_up_trims_username_email_and_password_before_persisting() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let input = new_user("  alice  ").with_email("  alice@example.com  ").with_password("  secret123  ");

    let user = service.sign_up(sign_up_user(input)).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].username, "alice");
    assert_eq!(created[0].email, "alice@example.com");
    assert_eq!(created[0].password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn sign_in_rejects_invalid_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::InvalidCredentials)));
}

#[tokio::test]
async fn sign_in_accepts_email_identifier() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service
        .sign_in(Credentials {
            identifier: "alice@example.com".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert_eq!(user.username, "alice");
    assert_eq!(repository.login_records(), vec![user_id(1)]);
}

#[tokio::test]
async fn sign_in_trims_identifier_and_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .sign_in(Credentials {
            identifier: "  alice  ".into(),
            password: "  secret123  ".into(),
        })
        .await
        .unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn sign_up_rejects_invalid_username_constraints() {
    for username in ["ab", "alice!", "-alice", "alice_"] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(sign_up_user(new_user(username))).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}

#[tokio::test]
async fn sign_up_rejects_invalid_password_constraints() {
    for password in ["short", ""] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(sign_up_user(new_user("alice").with_password(password))).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}

#[tokio::test]
async fn create_user_rejects_invalid_email_format() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.create_user(new_user("alice").with_email("not-an-email")).await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "email must be a valid email address"));
    assert_eq!(repository.created_records().len(), 0);
}

#[tokio::test]
async fn authenticated_user_returns_user_from_token_subject() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.authenticated_user(user_id(1)).await.unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn authenticated_user_rejects_unknown_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.authenticated_user(user_id(1)).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_user(new_user("alice")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn replace_user_allows_same_user_identity() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.replace_user(user_id(1), replace_user("alice", false)).await.unwrap();

    assert!(!user.is_active);
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn replace_user_keeps_existing_password_when_password_is_blank() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:existing"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let mut input = replace_user("alice", false);
    input.password = Some(String::new());

    let user = service.replace_user(user_id(1), input).await.unwrap();

    assert!(!user.is_active);
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:existing"));
}

#[tokio::test]
async fn replace_user_hashes_new_password_when_password_is_present() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:existing"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let mut input = replace_user("alice", false);
    input.password = Some("new-secret".into());

    service.replace_user(user_id(1), input).await.unwrap();

    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:new-secret"));
}

#[tokio::test]
async fn list_users_rejects_zero_page() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(PageRequest { page: 0, page_size: 10 }, Default::default()).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[tokio::test]
async fn list_users_rejects_page_size_above_maximum() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service
        .list_users(
            PageRequest {
                page: 1,
                page_size: MAX_PAGE_SIZE + 1,
            },
            Default::default(),
        )
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

pub(super) trait WithPassword {
    fn with_password(self, password: &str) -> Self;
    fn with_email(self, email: &str) -> Self;
}

impl WithPassword for NewUser {
    fn with_password(self, password: &str) -> Self {
        Self {
            password: password.into(),
            ..self
        }
    }

    fn with_email(self, email: &str) -> Self {
        Self { email: email.into(), ..self }
    }
}

fn sign_up_user(user: NewUser) -> SignUpUser {
    SignUpUser {
        user,
        email_verification_code: None,
        aff_code: None,
    }
}
