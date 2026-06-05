use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{
        AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse, IdentityProvider, NewUser,
        SignUpUser, USER_QUOTA_MODE_WALLET, User, UserId, UserIdentity, UserIdentityInput, UserListFilters, default_user_created_at,
    },
};
use user::application::{
    AffiliateRepository, AppError, AppResult, PasswordHasher, PasswordResetRecord, PasswordResetRepository, ReplaceUserRecord, UserAuthRecord, UserRepository,
    UserUseCase,
};

use super::{CachedUserRepository, ProxyCacheInvalidator};

#[tokio::test]
async fn signup_refreshes_scheduling_snapshot_from_repository_create() {
    let cache = RecordingInvalidator::default();
    let repository = CachedUserRepository::new(MemoryUserRepository::default(), cache.clone());
    let service = user::application::UserService::new(repository, TestPasswordHasher);

    service
        .sign_up(SignUpUser {
            user: new_user("demo"),
            email_verification_code: None,
            aff_code: None,
        })
        .await
        .unwrap();

    assert_eq!(cache.snapshot_refreshes(), 1);
    assert_eq!(cache.auth_bumps(), 0);
}

#[tokio::test]
async fn delete_user_bumps_auth_and_refreshes_scheduling() {
    let cache = RecordingInvalidator::default();
    let repository = CachedUserRepository::new(MemoryUserRepository::default(), cache.clone());

    repository.delete(UserId("user-1".into())).await.unwrap();

    assert_eq!(cache.auth_bumps(), 1);
    assert_eq!(cache.snapshot_refreshes(), 1);
}

#[derive(Clone, Default)]
struct RecordingInvalidator {
    state: Arc<Mutex<InvalidationState>>,
}

#[derive(Default)]
struct InvalidationState {
    snapshot_refreshes: usize,
    auth_bumps: usize,
}

impl RecordingInvalidator {
    fn snapshot_refreshes(&self) -> usize {
        self.state.lock().unwrap().snapshot_refreshes
    }

    fn auth_bumps(&self) -> usize {
        self.state.lock().unwrap().auth_bumps
    }
}

#[async_trait]
impl ProxyCacheInvalidator for RecordingInvalidator {
    async fn refresh_scheduling(&self) -> Result<(), crate::llm_proxy::LlmProxyError> {
        self.state.lock().unwrap().snapshot_refreshes += 1;
        Ok(())
    }

    async fn bump_auth(&self) -> Result<(), crate::llm_proxy::LlmProxyError> {
        self.state.lock().unwrap().auth_bumps += 1;
        Ok(())
    }

    async fn clear_provider_cooldown(&self, _provider_id: &str) -> Result<(), crate::llm_proxy::LlmProxyError> {
        Ok(())
    }
}

#[derive(Clone, Default)]
struct MemoryUserRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Default)]
struct RepositoryState {
    next_id: u64,
    users: Vec<StoredUser>,
    deleted: Vec<UserId>,
    identities: Vec<UserIdentity>,
}

struct StoredUser {
    user: User,
    password_hash: Option<String>,
}

#[async_trait]
impl UserRepository for MemoryUserRepository {
    async fn create(&self, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        state.next_id += 1;
        let user = user_from_record(state.next_id, &record);
        state.users.push(StoredUser {
            user: user.clone(),
            password_hash: record.password_hash,
        });
        Ok(user)
    }

    async fn replace(&self, id: UserId, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let stored = state.users.iter_mut().find(|stored| stored.user.id == id).ok_or(AppError::NotFound)?;
        stored.user = user_from_replace(id, &stored.user, &record);
        if let Some(password_hash) = record.password_hash {
            stored.password_hash = Some(password_hash);
        }
        Ok(stored.user.clone())
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.state.lock().unwrap().deleted.push(id);
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.id == id)
            .map(|stored| stored.user.clone()))
    }

    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.id == id)
            .map(StoredUser::auth_record))
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.email == email)
            .map(|stored| stored.user.clone()))
    }

    async fn find_by_affiliate_code(&self, affiliate_code: &str) -> AppResult<Option<User>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.affiliate_code == affiliate_code)
            .map(|stored| stored.user.clone()))
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.username == username)
            .map(StoredUser::auth_record))
    }

    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|stored| stored.user.email == email)
            .map(StoredUser::auth_record))
    }

    async fn record_login(&self, _id: UserId) -> AppResult<()> {
        Ok(())
    }

    async fn list(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        self.list_slice(
            PageSliceRequest {
                offset: (page.page - 1) * page.page_size,
                limit: page.page_size,
                page: page.page,
                page_size: page.page_size,
            },
            filters,
        )
        .await
    }

    async fn list_slice(&self, request: PageSliceRequest, _filters: UserListFilters) -> AppResult<Page<User>> {
        let users: Vec<User> = self.state.lock().unwrap().users.iter().map(|stored| stored.user.clone()).collect();
        Ok(Page {
            total: users.len() as u64,
            items: users.into_iter().skip(request.offset as usize).take(request.limit as usize).collect(),
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn create_identity(&self, input: UserIdentityInput) -> AppResult<UserIdentity> {
        let mut state = self.state.lock().unwrap();
        let identity = UserIdentity {
            id: format!("identity-{}", state.identities.len() + 1),
            user_id: input.user_id,
            provider: input.provider,
            provider_subject: input.provider_subject,
            email: input.email,
            email_verified: input.email_verified,
            display_name: input.display_name,
            avatar_url: input.avatar_url,
            created_at: default_user_created_at(),
            updated_at: default_user_created_at(),
            last_login_at: None,
        };
        state.identities.push(identity.clone());
        Ok(identity)
    }

    async fn find_identity(&self, provider: IdentityProvider, subject: &str) -> AppResult<Option<UserIdentity>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .identities
            .iter()
            .find(|identity| identity.provider == provider && identity.provider_subject == subject)
            .cloned())
    }

    async fn list_identities_by_user_id(&self, user_id: &str) -> AppResult<Vec<UserIdentity>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .identities
            .iter()
            .filter(|identity| identity.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn list_identities_by_user_ids(&self, user_ids: &[String]) -> AppResult<BTreeMap<String, Vec<UserIdentity>>> {
        let mut grouped = BTreeMap::<String, Vec<UserIdentity>>::new();
        for identity in self.state.lock().unwrap().identities.iter() {
            if user_ids.contains(&identity.user_id) {
                grouped.entry(identity.user_id.clone()).or_default().push(identity.clone());
            }
        }
        Ok(grouped)
    }

    async fn touch_identity_login(&self, _identity_id: &str) -> AppResult<()> {
        Ok(())
    }

    async fn delete_identity(&self, identity_id: &str) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();
        state.identities.retain(|identity| identity.id != identity_id);
        Ok(())
    }
}

#[async_trait]
impl AffiliateRepository for MemoryUserRepository {
    async fn affiliate_summary(&self, user_id: &str, affiliate_code: &str) -> AppResult<AffiliateSummaryResponse> {
        let referred_user_count = self
            .state
            .lock()
            .unwrap()
            .users
            .iter()
            .filter(|stored| stored.user.referred_by_user_id.as_ref().is_some_and(|id| id.0 == user_id))
            .count() as u64;
        Ok(AffiliateSummaryResponse {
            affiliate_code: affiliate_code.into(),
            affiliate_link: format!("/auth/sign-up?aff={affiliate_code}"),
            affiliate_enabled: false,
            referred_user_count,
            total_referred_recharge_amount: rust_decimal::Decimal::ZERO,
            total_commission_amount: rust_decimal::Decimal::ZERO,
            today_commission_amount: rust_decimal::Decimal::ZERO,
            month_commission_amount: rust_decimal::Decimal::ZERO,
            affiliate_commission_percent: rust_decimal::Decimal::ZERO,
            last_commission_at: None,
        })
    }

    async fn page_affiliate_referrals(
        &self,
        _user_id: &str,
        request: PageSliceRequest,
        _query: AffiliateReferralQuery,
    ) -> AppResult<Page<AffiliateReferralItem>> {
        Ok(empty_page(request))
    }

    async fn page_affiliate_commissions(
        &self,
        _user_id: &str,
        request: PageSliceRequest,
        _query: AffiliateCommissionQuery,
    ) -> AppResult<Page<AffiliateCommissionItem>> {
        Ok(empty_page(request))
    }

    async fn export_affiliate_commissions(&self, _user_id: &str, _query: AffiliateCommissionQuery) -> AppResult<Vec<AffiliateCommissionItem>> {
        Ok(Vec::new())
    }
}

fn empty_page<T>(request: PageSliceRequest) -> Page<T> {
    Page {
        items: Vec::new(),
        total: 0,
        page: request.page,
        page_size: request.page_size,
    }
}

#[async_trait]
impl PasswordResetRepository for MemoryUserRepository {
    async fn create_password_reset_token(&self, _record: PasswordResetRecord) -> AppResult<()> {
        Ok(())
    }

    async fn consume_password_reset_token(&self, _token_hash: &str, _password_hash: &str, _now: time::OffsetDateTime) -> AppResult<Option<User>> {
        Ok(None)
    }
}

impl StoredUser {
    fn auth_record(&self) -> UserAuthRecord {
        UserAuthRecord {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }
}

#[derive(Clone, Copy)]
struct TestPasswordHasher;

impl PasswordHasher for TestPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("hashed:{password}"))
    }
}

fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: "secret123".into(),
        email: format!("{username}@example.com"),
        role: "admin".into(),
        group_codes: None,
        is_active: true,
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_mode: USER_QUOTA_MODE_WALLET.into(),
        referrer_aff_code: None,
    }
}

fn user_from_record(id: u64, record: &ReplaceUserRecord) -> User {
    User {
        id: user_id(id),
        username: record.username.clone(),
        email: record.email.clone(),
        role: record.role.clone(),
        group_codes: record.group_codes.clone(),
        is_active: record.is_active,
        allowed_model_ids: record.allowed_model_ids.clone(),
        allowed_provider_ids: record.allowed_provider_ids.clone(),
        auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
        email_verified: record.email_verified.unwrap_or(false),
        password_set: record.password_hash.is_some(),
        system: false,
        rate_limit_rpm: record.rate_limit_rpm,
        quota_mode: record.quota_mode.clone(),
        affiliate_code: affiliate_code(id),
        referred_by_user_id: None,
        referred_at: None,
        created_at: default_user_created_at(),
        last_login_at: None,
    }
}

fn user_from_replace(id: UserId, current: &User, record: &ReplaceUserRecord) -> User {
    User {
        id,
        email_verified: record.email_verified.unwrap_or(current.email_verified),
        password_set: current.password_set || record.password_hash.is_some(),
        affiliate_code: current.affiliate_code.clone(),
        referred_by_user_id: current.referred_by_user_id.clone(),
        referred_at: current.referred_at.clone(),
        created_at: current.created_at.clone(),
        last_login_at: current.last_login_at.clone(),
        ..user_from_record(0, record)
    }
}

fn user_id(id: u64) -> UserId {
    UserId(format!("user-{id}"))
}

fn affiliate_code(id: u64) -> String {
    format!("aff-{id}")
}
