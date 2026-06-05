use types::user::SignUpUser;

use super::registration_email_test_support::{
    SavedCooldown, TestRegistrationEmailCodeStore, TestRegistrationEmailMailer, assert_invalid_input, email_code_request, service_with_registration_email,
    service_with_registration_email_repository,
};
use crate::{
    application::UserUseCase,
    test_support::{MemoryUserRepository, new_user},
};

#[tokio::test]
async fn request_registration_email_code_sets_cooldown_and_sends_code() {
    let store = TestRegistrationEmailCodeStore::default();
    let mailer = TestRegistrationEmailMailer::default();
    let service = service_with_registration_email(store.clone(), mailer.clone());

    service.request_registration_email_code(email_code_request("Alice@Example.com")).await.unwrap();

    let sent = mailer.sent();
    let saved = store.saved_codes();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].recipient_email, "alice@example.com");
    assert_eq!(sent[0].subject, "Your code");
    assert_eq!(sent[0].html, format!("Code {} expires in 10 minutes", saved[0].code));
    assert_eq!(saved[0].email, "alice@example.com");
    assert_eq!(saved[0].ttl_seconds, 600);
    assert_eq!(
        store.saved_cooldowns(),
        vec![SavedCooldown {
            email: "alice@example.com".into(),
            ttl_seconds: 60,
        }]
    );
}

#[tokio::test]
async fn request_registration_email_code_rejects_resend_inside_cooldown() {
    let store = TestRegistrationEmailCodeStore::default();
    let mailer = TestRegistrationEmailMailer::default();
    let service = service_with_registration_email(store, mailer.clone());

    service.request_registration_email_code(email_code_request("alice@example.com")).await.unwrap();
    let result = service.request_registration_email_code(email_code_request("alice@example.com")).await;

    assert_invalid_input(result, "registration email code can only be requested once every 60 seconds");
    assert_eq!(mailer.sent().len(), 1);
}

#[tokio::test]
async fn request_registration_email_code_reuses_unexpired_code_after_cooldown() {
    let store = TestRegistrationEmailCodeStore::default();
    let mailer = TestRegistrationEmailMailer::default();
    let service = service_with_registration_email(store.clone(), mailer.clone());

    service.request_registration_email_code(email_code_request("alice@example.com")).await.unwrap();
    store.clear_cooldown("alice@example.com");
    service.request_registration_email_code(email_code_request("alice@example.com")).await.unwrap();

    let sent = mailer.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].html, sent[1].html);
    assert_eq!(store.saved_codes().len(), 1);
}

#[tokio::test]
async fn sign_up_consumes_registration_email_code_once() {
    let repository = MemoryUserRepository::default();
    let store = TestRegistrationEmailCodeStore::default();
    store.seed_code("alice@example.com", "123456");
    let service = service_with_registration_email_repository(repository.clone(), store);

    service
        .sign_up(SignUpUser {
            user: new_user("alice"),
            email_verification_code: Some("123456".into()),
            aff_code: None,
        })
        .await
        .unwrap();

    let result = service
        .sign_up(SignUpUser {
            user: new_user("bob"),
            email_verification_code: Some("123456".into()),
            aff_code: None,
        })
        .await;
    assert_invalid_input(result, "email verification code is invalid or expired");
    assert_eq!(repository.created_records().len(), 1);
}
