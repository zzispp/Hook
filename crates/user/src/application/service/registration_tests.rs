use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    system_setting::EmailSuffixMode,
    user::{NewUser, SignUpUser},
};

use super::{NoInitialGrantLedger, NoSystemUserProvider, NoUserWalletCatalog};
use crate::{
    application::{AppError, RegistrationPolicy, RegistrationSettings, UserService, UserUseCase},
    test_support::{MemoryUserRepository, TestPasswordHasher, new_user},
};

type TestRegistrationService =
    UserService<MemoryUserRepository, TestPasswordHasher, NoSystemUserProvider, TestRegistrationPolicy, NoInitialGrantLedger, NoUserWalletCatalog>;

#[derive(Clone)]
struct TestRegistrationPolicy {
    settings: RegistrationSettings,
}

#[tokio::test]
async fn sign_up_allows_whitelisted_email_suffix() {
    let repository = MemoryUserRepository::default();
    let service = service_with_suffix_policy(repository.clone(), EmailSuffixMode::Whitelist, "example.com, company.com");
    let input = user_with_email("alice", "  Alice@Example.COM  ");

    let user = service.sign_up(sign_up_user(input)).await.unwrap();

    assert_eq!(user.username, "alice");
    assert_eq!(repository.created_records().len(), 1);
}

#[tokio::test]
async fn sign_up_rejects_email_outside_whitelist() {
    let repository = MemoryUserRepository::default();
    let service = service_with_suffix_policy(repository.clone(), EmailSuffixMode::Whitelist, "example.com");

    let result = service.sign_up(sign_up_user(user_with_email("alice", "alice@blocked.com"))).await;

    assert_invalid_input(result, "email suffix is not allowed for new users");
    assert_eq!(repository.created_records().len(), 0);
}

#[tokio::test]
async fn sign_up_rejects_blacklisted_email_suffix() {
    let repository = MemoryUserRepository::default();
    let service = service_with_suffix_policy(repository.clone(), EmailSuffixMode::Blacklist, "example.com");

    let result = service.sign_up(sign_up_user(new_user("alice"))).await;

    assert_invalid_input(result, "email suffix is not allowed for new users");
    assert_eq!(repository.created_records().len(), 0);
}

#[tokio::test]
async fn create_user_rejects_email_outside_whitelist() {
    let repository = MemoryUserRepository::default();
    let service = service_with_suffix_policy(repository.clone(), EmailSuffixMode::Whitelist, "example.com");

    let result = service.create_user(user_with_email("alice", "alice@blocked.com")).await;

    assert_invalid_input(result, "email suffix is not allowed for new users");
    assert_eq!(repository.created_records().len(), 0);
}

#[tokio::test]
async fn create_user_ignores_closed_self_registration() {
    let repository = MemoryUserRepository::default();
    let service = service_with_registration_settings(
        repository.clone(),
        RegistrationSettings {
            allow_registration: false,
            registration_email_verification_enabled: false,
            default_user_grant: Decimal::ZERO,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: EmailSuffixMode::None,
            email_suffixes: String::new(),
        },
    );

    let user = service.create_user(new_user("alice")).await.unwrap();

    assert_eq!(user.username, "alice");
    assert_eq!(repository.created_records().len(), 1);
}

fn service_with_suffix_policy(repository: MemoryUserRepository, mode: EmailSuffixMode, suffixes: &str) -> TestRegistrationService {
    service_with_registration_settings(
        repository,
        RegistrationSettings {
            allow_registration: true,
            registration_email_verification_enabled: false,
            default_user_grant: Decimal::ZERO,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: mode,
            email_suffixes: suffixes.into(),
        },
    )
}

fn service_with_registration_settings(repository: MemoryUserRepository, settings: RegistrationSettings) -> TestRegistrationService {
    UserService::with_system_user_and_registration(
        repository,
        TestPasswordHasher,
        NoSystemUserProvider,
        TestRegistrationPolicy { settings },
        NoInitialGrantLedger,
        NoUserWalletCatalog,
    )
}

fn user_with_email(username: &str, email: &str) -> NewUser {
    NewUser {
        email: email.into(),
        ..new_user(username)
    }
}

fn sign_up_user(user: NewUser) -> SignUpUser {
    SignUpUser {
        user,
        email_verification_code: None,
    }
}

fn assert_invalid_input<T>(result: Result<T, AppError>, expected: &str) {
    match result {
        Err(AppError::InvalidInput(message)) => assert_eq!(message, expected),
        Err(error) => panic!("expected invalid input error, got {error:?}"),
        Ok(_) => panic!("expected invalid input error, got ok"),
    }
}

#[async_trait]
impl RegistrationPolicy for TestRegistrationPolicy {
    async fn registration_settings(&self) -> crate::application::AppResult<RegistrationSettings> {
        Ok(self.settings.clone())
    }
}
