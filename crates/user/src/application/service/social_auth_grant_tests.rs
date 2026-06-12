use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use types::{system_setting::EmailSuffixMode, user::IdentityProvider};

use super::social_auth_test_support::{
    TestAuthProviderConfig, TestAuthTicketStore, TestEmailConfig, TestMailer, TestOAuthClient, TestPurposeEmailCodeStore, github_profile, state_from_url,
};
use crate::{
    application::{AppResult, InitialGrantLedger, OAuthSignInResult, RegistrationPolicy, RegistrationSettings, UserService, UserUseCase},
    test_support::{MemoryUserRepository, TestPasswordHasher},
};

#[derive(Clone, Default)]
struct RecordingInitialGrantLedger {
    grants: Arc<Mutex<Vec<(String, Decimal)>>>,
}

#[derive(Clone)]
struct GrantRegistrationPolicy {
    default_user_grant: Decimal,
}

#[tokio::test]
async fn oauth_first_account_creation_grants_default_user_balance() {
    let repository = MemoryUserRepository::default();
    let ledger = RecordingInitialGrantLedger::default();
    let amount = Decimal::new(1250, 2);
    let service = service_with_grant(
        repository,
        ledger.clone(),
        amount,
        TestOAuthClient::with_profile(github_profile("new@example.com")),
    );

    let state = state_from_url(&service.oauth_start(IdentityProvider::Github, None).await.unwrap());
    let result = service.oauth_callback(IdentityProvider::Github, "oauth-code".into(), state).await.unwrap();

    let OAuthSignInResult::Authenticated(user) = result else {
        panic!("expected authenticated OAuth result");
    };
    assert_eq!(ledger.grants(), vec![(user.id.0, amount)]);
}

fn service_with_grant(
    repository: MemoryUserRepository,
    ledger: RecordingInitialGrantLedger,
    amount: Decimal,
    oauth_client: TestOAuthClient,
) -> impl UserUseCase {
    UserService::with_system_user_and_registration(
        repository,
        TestPasswordHasher,
        super::NoSystemUserProvider,
        GrantRegistrationPolicy { default_user_grant: amount },
        ledger,
        super::NoUserWalletCatalog,
    )
    .with_registration_email(TestEmailConfig, TestMailer::default(), super::NoRegistrationEmailCodeStore)
    .with_social_auth(
        TestAuthProviderConfig::default(),
        oauth_client,
        TestAuthTicketStore::default(),
        TestPurposeEmailCodeStore::default(),
    )
}

impl RecordingInitialGrantLedger {
    fn grants(&self) -> Vec<(String, Decimal)> {
        self.grants.lock().unwrap().clone()
    }
}

#[async_trait]
impl InitialGrantLedger for RecordingInitialGrantLedger {
    async fn grant_initial_balance(&self, user_id: &str, amount: Decimal) -> AppResult<()> {
        self.grants.lock().unwrap().push((user_id.to_owned(), amount));
        Ok(())
    }
}

#[async_trait]
impl RegistrationPolicy for GrantRegistrationPolicy {
    async fn registration_settings(&self) -> AppResult<RegistrationSettings> {
        Ok(RegistrationSettings {
            allow_registration: true,
            registration_email_verification_enabled: false,
            default_user_grant: self.default_user_grant,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: EmailSuffixMode::None,
            email_suffixes: String::new(),
        })
    }
}
