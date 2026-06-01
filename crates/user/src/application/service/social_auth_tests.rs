use types::user::{AccountPasswordChangePayload, AccountPasswordEmailCodePayload, Credentials, IdentityProvider};

use super::social_auth_test_support::{
    TestAuthTicketStore, TestOAuthClient, TestPurposeEmailCodeStore, github_profile, identity_input, redirect_uri, state_from_url, test_service,
    test_service_with_codes, test_service_with_system_user, test_service_with_tickets,
};
use crate::{
    application::{AppError, AuthTicketStore, OAuthProfile, OAuthSignInResult, UserService, UserUseCase, WalletSignInInput},
    test_support::{MemoryUserRepository, TestPasswordHasher, passwordless_stored_user, stored_user, user_id},
};

#[tokio::test]
async fn oauth_verified_email_without_user_creates_passwordless_user() {
    let repository = MemoryUserRepository::default();
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("new@example.com")));

    let state = oauth_state(&service).await;
    let result = service.oauth_callback(IdentityProvider::Github, "oauth-code".into(), state).await.unwrap();

    let OAuthSignInResult::Authenticated(user) = result else {
        panic!("expected authenticated OAuth result");
    };
    assert_eq!(user.email, "new@example.com");
    assert!(!user.password_set);
    assert_eq!(repository.created_records()[0].password_hash, None);
    assert_eq!(repository.identities()[0].provider, IdentityProvider::Github);
}

#[tokio::test]
async fn oauth_start_uses_public_base_url_callback() {
    let service = test_service(MemoryUserRepository::default(), TestOAuthClient::default());

    let url = service.oauth_start(IdentityProvider::Github).await.unwrap();
    let encoded_redirect = redirect_uri().replace(':', "%3A").replace('/', "%2F");

    assert!(url.contains(&format!("redirect_uri={encoded_redirect}")));
}

#[tokio::test]
async fn oauth_existing_email_requires_binding_ticket_then_binds() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("alice@example.com")));

    let result = oauth_callback(&service).await;
    let OAuthSignInResult::BindingRequired { ticket, email, username, .. } = result else {
        panic!("expected OAuth binding ticket");
    };

    assert_eq!(email, "alice@example.com");
    assert_eq!(username, "alice");
    assert!(repository.identities().is_empty());
    let user = service.bind_oauth_existing(IdentityProvider::Github, ticket).await.unwrap();

    assert_eq!(user.id, user_id(1));
    assert_eq!(repository.identities()[0].user_id, user_id(1).0);
}

#[tokio::test]
async fn account_oauth_callback_links_identity_to_current_user() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("other@example.com")));

    let state = account_oauth_state(&service, user_id(1)).await;
    let result = service
        .account_oauth_callback(user_id(1), IdentityProvider::Github, "oauth-code".into(), state)
        .await
        .unwrap();

    assert_eq!(result.identity.provider, IdentityProvider::Github);
    assert_eq!(repository.users().len(), 1);
    assert_eq!(repository.identities()[0].user_id, user_id(1).0);
}

#[tokio::test]
async fn account_oauth_callback_rejects_existing_identity_subject() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    repository.seed_user(stored_user(2, "bob", "hashed:secret123"));
    repository.seed_identity(identity_input(user_id(2).0, IdentityProvider::Github, "github-subject"));
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("alice@example.com")));

    let result = service
        .account_oauth_callback(
            user_id(1),
            IdentityProvider::Github,
            "oauth-code".into(),
            account_oauth_state(&service, user_id(1)).await,
        )
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "provider identity is already linked"));
    assert_eq!(repository.identities().len(), 1);
}

#[tokio::test]
async fn sign_in_oauth_callback_rejects_account_binding_state() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("alice@example.com")));

    let result = service
        .oauth_callback(IdentityProvider::Github, "oauth-code".into(), account_oauth_state(&service, user_id(1)).await)
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert!(repository.identities().is_empty());
}

#[tokio::test]
async fn oauth_binding_ticket_rejects_wrong_provider_path() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = test_service(repository.clone(), TestOAuthClient::with_profile(github_profile("alice@example.com")));

    let result = oauth_callback(&service).await;
    let OAuthSignInResult::BindingRequired { ticket, .. } = result else {
        panic!("expected OAuth binding ticket");
    };
    let result = service.bind_oauth_existing(IdentityProvider::Google, ticket).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert!(repository.identities().is_empty());
}

#[tokio::test]
async fn oauth_rejects_unverified_provider_email() {
    let profile = OAuthProfile {
        email_verified: false,
        ..github_profile("alice@example.com")
    };
    let service = test_service(MemoryUserRepository::default(), TestOAuthClient::with_profile(profile));

    let result = service
        .oauth_callback(IdentityProvider::Github, "oauth-code".into(), oauth_state(&service).await)
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "verified provider email is required"));
}

#[tokio::test]
async fn oauth_rejects_system_user_email() {
    let repository = MemoryUserRepository::default();
    let service = test_service_with_system_user(
        repository.clone(),
        TestPurposeEmailCodeStore::default(),
        TestAuthTicketStore::default(),
        TestOAuthClient::with_profile(github_profile("admin@example.com")),
    );

    let result = service
        .oauth_callback(IdentityProvider::Github, "oauth-code".into(), oauth_state(&service).await)
        .await;

    assert_system_self_service_error(result);
    assert!(repository.created_records().is_empty());
    assert!(repository.identities().is_empty());
}

#[tokio::test]
async fn passwordless_user_cannot_sign_in_with_password() {
    let service = UserService::new(MemoryUserRepository::with_user(passwordless_stored_user(1, "alice")), TestPasswordHasher);

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "secret123".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::PasswordNotSet)));
}

#[tokio::test]
async fn unlink_rejects_last_login_method_for_passwordless_user() {
    let repository = MemoryUserRepository::with_user(passwordless_stored_user(1, "alice"));
    repository.seed_identity(identity_input(user_id(1).0, IdentityProvider::Github, "github-1"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.unlink_identity(user_id(1), "identity-1".into()).await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "at least one login method must remain"));
    assert_eq!(repository.identities().len(), 1);
}

#[tokio::test]
async fn account_password_change_sets_local_password_after_email_code() {
    let repository = MemoryUserRepository::with_user(passwordless_stored_user(1, "alice"));
    let codes = TestPurposeEmailCodeStore::default();
    let service = test_service_with_codes(repository.clone(), codes.clone(), TestOAuthClient::default());

    service
        .request_account_password_email_code(user_id(1), AccountPasswordEmailCodePayload { lang: "en".into() })
        .await
        .unwrap();
    let user = service
        .change_account_password(
            user_id(1),
            AccountPasswordChangePayload {
                email_verification_code: codes.saved_code("account_password", "alice@example.com"),
                password: "new-secret123".into(),
            },
        )
        .await
        .unwrap();

    assert!(user.password_set);
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:new-secret123"));
}

#[tokio::test]
async fn wallet_ticket_complete_creates_passwordless_user() {
    let repository = MemoryUserRepository::default();
    let codes = TestPurposeEmailCodeStore::default();
    let tickets = TestAuthTicketStore::default();
    let service = test_service_with_tickets(repository.clone(), codes.clone(), tickets.clone());
    tickets
        .seed_wallet_binding("wallet-ticket", identity_input(String::new(), IdentityProvider::Evm, "0xabc"))
        .await;
    codes.seed_code("wallet_binding", "wallet@example.com", "123456");

    let user = service
        .complete_wallet("wallet-ticket".into(), "wallet@example.com".into(), "123456".into())
        .await
        .unwrap();

    assert_eq!(user.email, "wallet@example.com");
    assert!(!user.password_set);
    assert_eq!(repository.identities()[0].provider, IdentityProvider::Evm);
}

#[tokio::test]
async fn wallet_email_code_rejects_system_user_email() {
    let repository = MemoryUserRepository::default();
    let codes = TestPurposeEmailCodeStore::default();
    let tickets = TestAuthTicketStore::default();
    let service = test_service_with_system_user(repository, codes.clone(), tickets.clone(), TestOAuthClient::default());
    tickets
        .seed_wallet_binding("wallet-ticket", identity_input(String::new(), IdentityProvider::Evm, "0xabc"))
        .await;

    let result = service
        .request_wallet_email_code("wallet-ticket".into(), "admin@example.com".into(), "en".into())
        .await;

    assert_system_self_service_error(result);
}

#[tokio::test]
async fn wallet_complete_rejects_system_user_email() {
    let repository = MemoryUserRepository::default();
    let codes = TestPurposeEmailCodeStore::default();
    let tickets = TestAuthTicketStore::default();
    let service = test_service_with_system_user(repository.clone(), codes.clone(), tickets.clone(), TestOAuthClient::default());
    tickets
        .seed_wallet_binding("wallet-ticket", identity_input(String::new(), IdentityProvider::Evm, "0xabc"))
        .await;
    codes.seed_code("wallet_binding", "admin@example.com", "123456");

    let result = service
        .complete_wallet("wallet-ticket".into(), "admin@example.com".into(), "123456".into())
        .await;

    assert_system_self_service_error(result);
    assert!(repository.created_records().is_empty());
    assert!(repository.identities().is_empty());
}

#[tokio::test]
async fn account_wallet_link_rejects_existing_wallet_identity() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    repository.seed_user(stored_user(2, "bob", "hashed:secret123"));
    repository.seed_identity(identity_input(user_id(2).0, IdentityProvider::Evm, "0xabc"));
    let tickets = TestAuthTicketStore::default();
    tickets
        .save_wallet_challenge(
            "testnonce",
            crate::application::WalletChallenge {
                provider: IdentityProvider::Evm,
                address: "0xabc".into(),
                nonce: "testnonce".into(),
                message: "testnonce".into(),
                chain_id: Some(1),
            },
            600,
        )
        .await
        .unwrap();
    let service = test_service_with_tickets(repository.clone(), TestPurposeEmailCodeStore::default(), tickets);

    let result = service
        .account_wallet_link(
            user_id(1),
            WalletSignInInput {
                provider: IdentityProvider::Evm,
                address: "0xabc".into(),
                message: "testnonce".into(),
                signature: "ignored".into(),
                chain_id: Some(1),
            },
        )
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert_eq!(repository.identities().len(), 1);
}

#[tokio::test]
async fn wallet_existing_identity_signs_in_without_email() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    repository.seed_identity(identity_input(user_id(1).0, IdentityProvider::Evm, "0xabc"));
    let tickets = TestAuthTicketStore::default();
    tickets
        .save_wallet_challenge(
            "testnonce",
            crate::application::WalletChallenge {
                provider: IdentityProvider::Evm,
                address: "0xabc".into(),
                nonce: "testnonce".into(),
                message: "testnonce".into(),
                chain_id: Some(1),
            },
            600,
        )
        .await
        .unwrap();
    let service = test_service_with_tickets(repository, TestPurposeEmailCodeStore::default(), tickets);

    let result = service
        .wallet_sign_in(WalletSignInInput {
            provider: IdentityProvider::Evm,
            address: "0xabc".into(),
            message: "testnonce".into(),
            signature: "ignored".into(),
            chain_id: Some(1),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

async fn oauth_state(service: &impl UserUseCase) -> String {
    state_from_url(&service.oauth_start(IdentityProvider::Github).await.unwrap())
}

async fn account_oauth_state(service: &impl UserUseCase, user_id: types::user::UserId) -> String {
    state_from_url(&service.account_oauth_start(user_id, IdentityProvider::Github).await.unwrap())
}

async fn oauth_callback(service: &impl UserUseCase) -> OAuthSignInResult {
    service
        .oauth_callback(IdentityProvider::Github, "oauth-code".into(), oauth_state(service).await)
        .await
        .unwrap()
}

fn assert_system_self_service_error<T>(result: Result<T, AppError>) {
    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message == "system user cannot use account self-service"));
}
