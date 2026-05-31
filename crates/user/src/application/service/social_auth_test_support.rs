use async_trait::async_trait;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use types::user::{IdentityProvider, UserIdentityInput};

use crate::{
    application::{
        AppError, AppResult, AuthProviderConfig, AuthTicketStore, OAuthClient, OAuthPendingBinding, OAuthProfile, OAuthProviderSettings, OAuthStateRecord,
        UserService, WalletChallenge, WalletPendingBinding, WalletProviderSettings,
    },
    test_support::{MemoryUserRepository, TestPasswordHasher},
};

pub(super) use super::social_auth_email_test_support::{TestEmailConfig, TestMailer, TestPurposeEmailCodeStore, TestRegistrationPolicy};

type TestService = UserService<
    MemoryUserRepository,
    TestPasswordHasher,
    super::NoSystemUserProvider,
    TestRegistrationPolicy,
    super::NoInitialGrantLedger,
    super::NoUserWalletCatalog,
    super::NoPasswordResetConfig,
    super::NoPasswordResetMailer,
    TestEmailConfig,
    TestMailer,
    super::NoRegistrationEmailCodeStore,
    TestAuthProviderConfig,
    TestOAuthClient,
    TestAuthTicketStore,
    TestPurposeEmailCodeStore,
>;

#[derive(Clone)]
pub(super) struct TestAuthProviderConfig {
    oauth: OAuthProviderSettings,
    wallet: WalletProviderSettings,
}

#[derive(Clone, Default)]
pub(super) struct TestOAuthClient {
    profile: Option<OAuthProfile>,
}

#[derive(Clone, Default)]
pub(super) struct TestAuthTicketStore {
    state: Arc<Mutex<TicketState>>,
}

#[derive(Default)]
struct TicketState {
    oauth_states: BTreeMap<String, OAuthStateRecord>,
    oauth_bindings: BTreeMap<String, OAuthPendingBinding>,
    wallet_challenges: BTreeMap<String, WalletChallenge>,
    wallet_bindings: BTreeMap<String, WalletPendingBinding>,
}

pub(super) fn test_service(repository: MemoryUserRepository, oauth_client: TestOAuthClient) -> TestService {
    test_service_with_codes(repository, TestPurposeEmailCodeStore::default(), oauth_client)
}

pub(super) fn test_service_with_codes(repository: MemoryUserRepository, codes: TestPurposeEmailCodeStore, oauth_client: TestOAuthClient) -> TestService {
    UserService::with_system_user_and_registration(
        repository,
        TestPasswordHasher,
        super::NoSystemUserProvider,
        TestRegistrationPolicy,
        super::NoInitialGrantLedger,
        super::NoUserWalletCatalog,
    )
    .with_registration_email(TestEmailConfig, TestMailer::default(), super::NoRegistrationEmailCodeStore)
    .with_social_auth(TestAuthProviderConfig::default(), oauth_client, TestAuthTicketStore::default(), codes)
}

pub(super) fn redirect_uri() -> String {
    "https://app.example.com/auth/oauth/callback/github".into()
}

pub(super) fn state_from_url(url: &str) -> String {
    url.split("state=").nth(1).unwrap().split('&').next().unwrap().to_owned()
}

pub(super) fn github_profile(email: &str) -> OAuthProfile {
    OAuthProfile {
        subject: "github-subject".into(),
        email: email.into(),
        email_verified: true,
        display_name: Some("GitHub User".into()),
        avatar_url: Some("https://avatars.example.com/u/1".into()),
        metadata_json: "{}".into(),
    }
}

pub(super) fn identity_input(user_id: String, provider: IdentityProvider, subject: &str) -> UserIdentityInput {
    UserIdentityInput {
        user_id,
        provider,
        provider_subject: subject.into(),
        email: None,
        email_verified: false,
        display_name: None,
        avatar_url: None,
        metadata_json: "{}".into(),
    }
}

impl Default for TestAuthProviderConfig {
    fn default() -> Self {
        Self {
            oauth: OAuthProviderSettings {
                enabled: true,
                client_id: "client-id".into(),
                client_secret: "client-secret".into(),
                public_base_url: "https://app.example.com".into(),
            },
            wallet: WalletProviderSettings {
                evm_enabled: true,
                evm_chain_ids: vec![1],
                evm_statement: "Sign in to Hook".into(),
                domain: "app.example.com".into(),
            },
        }
    }
}

impl TestOAuthClient {
    pub(super) fn with_profile(profile: OAuthProfile) -> Self {
        Self { profile: Some(profile) }
    }
}

impl TestAuthTicketStore {
    pub(super) async fn seed_wallet_binding(&self, ticket: &str, identity: UserIdentityInput) {
        self.save_wallet_binding(ticket, WalletPendingBinding { identity }, 600).await.unwrap();
    }
}

#[async_trait]
impl AuthProviderConfig for TestAuthProviderConfig {
    async fn oauth_provider_settings(&self, _provider: IdentityProvider) -> AppResult<OAuthProviderSettings> {
        Ok(self.oauth.clone())
    }

    async fn wallet_provider_settings(&self) -> AppResult<WalletProviderSettings> {
        Ok(self.wallet.clone())
    }
}

#[async_trait]
impl OAuthClient for TestOAuthClient {
    async fn fetch_profile(&self, _provider: IdentityProvider, _settings: OAuthProviderSettings, _code: &str, _redirect_uri: &str) -> AppResult<OAuthProfile> {
        self.profile
            .clone()
            .ok_or_else(|| AppError::Infrastructure("test OAuth profile is missing".into()))
    }
}

#[async_trait]
impl AuthTicketStore for TestAuthTicketStore {
    async fn save_oauth_state(&self, state: &str, record: OAuthStateRecord, _ttl_seconds: u64) -> AppResult<()> {
        self.state.lock().unwrap().oauth_states.insert(state.into(), record);
        Ok(())
    }

    async fn consume_oauth_state(&self, state: &str) -> AppResult<Option<OAuthStateRecord>> {
        Ok(self.state.lock().unwrap().oauth_states.remove(state))
    }

    async fn save_oauth_binding(&self, ticket: &str, record: OAuthPendingBinding, _ttl_seconds: u64) -> AppResult<()> {
        self.state.lock().unwrap().oauth_bindings.insert(ticket.into(), record);
        Ok(())
    }

    async fn consume_oauth_binding(&self, ticket: &str) -> AppResult<Option<OAuthPendingBinding>> {
        Ok(self.state.lock().unwrap().oauth_bindings.remove(ticket))
    }

    async fn save_wallet_challenge(&self, nonce: &str, record: WalletChallenge, _ttl_seconds: u64) -> AppResult<()> {
        self.state.lock().unwrap().wallet_challenges.insert(nonce.into(), record);
        Ok(())
    }

    async fn consume_wallet_challenge(&self, nonce: &str) -> AppResult<Option<WalletChallenge>> {
        Ok(self.state.lock().unwrap().wallet_challenges.remove(nonce))
    }

    async fn save_wallet_binding(&self, ticket: &str, record: WalletPendingBinding, _ttl_seconds: u64) -> AppResult<()> {
        self.state.lock().unwrap().wallet_bindings.insert(ticket.into(), record);
        Ok(())
    }

    async fn get_wallet_binding(&self, ticket: &str) -> AppResult<Option<WalletPendingBinding>> {
        Ok(self.state.lock().unwrap().wallet_bindings.get(ticket).cloned())
    }

    async fn consume_wallet_binding(&self, ticket: &str) -> AppResult<Option<WalletPendingBinding>> {
        Ok(self.state.lock().unwrap().wallet_bindings.remove(ticket))
    }
}
