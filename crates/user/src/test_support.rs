use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateOverviewResponse, AdminAffiliateRelationChangeItem,
        AdminAffiliateRelationChangeQuery, AdminAffiliateRelationItem, AdminAffiliateRelationQuery, AdminAffiliateReportQuery, AdminAffiliateReportResponse,
        AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateRelationChangeRecord,
        AffiliateSummaryResponse, IdentityProvider, NewUser, ReplaceUser, USER_QUOTA_MODE_WALLET, User, UserId, UserIdentity, UserIdentityInput,
        UserListFilters, default_user_created_at,
    },
};

use crate::application::{
    AdminAffiliateRepository, AffiliateRelationUpdateRecord, AffiliateRepository, AppError, AppResult, PasswordHasher, PasswordResetRecord,
    PasswordResetRepository, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserRepository,
};

pub(crate) const VALID_PASSWORD: &str = "secret123";

#[derive(Clone, Default)]
pub(crate) struct MemoryUserRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Default)]
struct RepositoryState {
    next_id: u64,
    users: Vec<StoredUser>,
    created: Vec<ReplaceUserRecord>,
    replaced: Vec<(UserId, ReplaceUserRecord)>,
    deleted: Vec<UserId>,
    logins: Vec<UserId>,
    reset_tokens: Vec<StoredPasswordResetToken>,
    identities: Vec<UserIdentity>,
}

#[derive(Clone)]
pub(crate) struct StoredUser {
    user: User,
    password_hash: Option<String>,
}

#[derive(Clone)]
struct StoredPasswordResetToken {
    user_id: UserId,
    token_hash: String,
    expires_at: time::OffsetDateTime,
    consumed_at: Option<time::OffsetDateTime>,
}

#[derive(Clone)]
pub(crate) struct TestPasswordHasher;

#[derive(Clone)]
pub(crate) struct TestSystemUserProvider {
    record: SystemUserRecord,
}

impl MemoryUserRepository {
    pub(crate) fn with_user(user: StoredUser) -> Self {
        let repository = Self::default();
        repository.state.lock().unwrap().users.push(user);
        repository
    }

    pub(crate) fn with_users(users: Vec<StoredUser>) -> Self {
        let repository = Self::default();
        repository.state.lock().unwrap().users = users;
        repository
    }

    pub(crate) fn seed_user(&self, user: StoredUser) {
        self.state.lock().unwrap().users.push(user);
    }

    pub(crate) fn users(&self) -> Vec<User> {
        self.state.lock().unwrap().users.iter().map(|item| item.user.clone()).collect()
    }

    pub(crate) fn created_records(&self) -> Vec<ReplaceUserRecord> {
        self.state.lock().unwrap().created.clone()
    }

    pub(crate) fn replaced_records(&self) -> Vec<(UserId, ReplaceUserRecord)> {
        self.state.lock().unwrap().replaced.clone()
    }

    pub(crate) fn deleted_records(&self) -> Vec<UserId> {
        self.state.lock().unwrap().deleted.clone()
    }

    pub(crate) fn login_records(&self) -> Vec<UserId> {
        self.state.lock().unwrap().logins.clone()
    }

    pub(crate) fn seed_identity(&self, input: UserIdentityInput) -> UserIdentity {
        let mut state = self.state.lock().unwrap();
        let identity = identity_from_input(format!("identity-{}", state.identities.len() + 1), input);
        state.identities.push(identity.clone());
        identity
    }

    pub(crate) fn identities(&self) -> Vec<UserIdentity> {
        self.state.lock().unwrap().identities.clone()
    }
}

#[async_trait]
impl UserRepository for MemoryUserRepository {
    async fn create(&self, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let id = next_user_id(&mut state);
        let referred_by_user_id = referrer_user_id(&state, record.referrer_aff_code.as_deref())?;
        let user = user_from_record(id, &record, referred_by_user_id);
        state.users.push(StoredUser {
            user: user.clone(),
            password_hash: record.password_hash.clone(),
        });
        state.created.push(record);
        Ok(user)
    }

    async fn replace(&self, id: UserId, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let user = replace_stored_user(&mut state, &id, &record)?;
        state.replaced.push((id, record));
        Ok(user)
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

    async fn record_login(&self, id: UserId) -> AppResult<()> {
        self.state.lock().unwrap().logins.push(id);
        Ok(())
    }

    async fn list(&self, page: PageRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        let request = PageSliceRequest {
            offset: (page.page - 1) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(request, filters).await
    }

    async fn list_slice(&self, request: PageSliceRequest, filters: UserListFilters) -> AppResult<Page<User>> {
        let state = self.state.lock().unwrap();
        let users: Vec<User> = state
            .users
            .iter()
            .map(|stored| stored.user.clone())
            .filter(|user| user_matches_filters(user, &filters))
            .collect();
        let start = request.offset as usize;
        let end = start.saturating_add(request.limit as usize).min(users.len());
        let items = if start >= users.len() { vec![] } else { users[start..end].to_vec() };
        Ok(Page {
            items,
            total: users.len() as u64,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn create_identity(&self, input: UserIdentityInput) -> AppResult<UserIdentity> {
        let mut state = self.state.lock().unwrap();
        let identity = identity_from_input(format!("identity-{}", state.identities.len() + 1), input);
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

    async fn touch_identity_login(&self, identity_id: &str) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();
        let identity = state
            .identities
            .iter_mut()
            .find(|identity| identity.id == identity_id)
            .ok_or(AppError::NotFound)?;
        identity.last_login_at = Some(default_user_created_at());
        Ok(())
    }

    async fn delete_identity(&self, identity_id: &str) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();
        let current_len = state.identities.len();
        state.identities.retain(|identity| identity.id != identity_id);
        if state.identities.len() == current_len {
            return Err(AppError::NotFound);
        }
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

#[async_trait]
impl PasswordResetRepository for MemoryUserRepository {
    async fn create_password_reset_token(&self, record: PasswordResetRecord) -> AppResult<()> {
        self.state.lock().unwrap().reset_tokens.push(StoredPasswordResetToken {
            user_id: record.user_id,
            token_hash: record.token_hash,
            expires_at: record.expires_at,
            consumed_at: None,
        });
        Ok(())
    }

    async fn consume_password_reset_token(&self, token_hash: &str, password_hash: &str, now: time::OffsetDateTime) -> AppResult<Option<User>> {
        let mut state = self.state.lock().unwrap();
        let Some(index) = state.reset_tokens.iter().position(|token| token.token_hash == token_hash) else {
            return Ok(None);
        };
        if state.reset_tokens[index].consumed_at.is_some() || state.reset_tokens[index].expires_at <= now {
            return Ok(None);
        }
        let user_id = state.reset_tokens[index].user_id.clone();
        let stored = find_stored_user_mut(&mut state, &user_id)?;
        stored.password_hash = Some(password_hash.to_owned());
        stored.user.password_set = true;
        let user = stored.user.clone();
        state.reset_tokens[index].consumed_at = Some(now);
        Ok(Some(user))
    }
}

#[async_trait]
impl AdminAffiliateRepository for MemoryUserRepository {
    async fn admin_affiliate_overview(&self) -> AppResult<AdminAffiliateOverviewResponse> {
        Ok(AdminAffiliateOverviewResponse {
            total_referred_users: 0,
            active_referrer_count: 0,
            total_commission_amount: rust_decimal::Decimal::ZERO,
            today_commission_amount: rust_decimal::Decimal::ZERO,
            month_commission_amount: rust_decimal::Decimal::ZERO,
            affiliate_commission_percent: rust_decimal::Decimal::ZERO,
        })
    }

    async fn page_admin_affiliate_relations(
        &self,
        request: PageSliceRequest,
        _query: AdminAffiliateRelationQuery,
    ) -> AppResult<Page<AdminAffiliateRelationItem>> {
        Ok(empty_page(request))
    }

    async fn update_affiliate_relation(&self, user_id: &str, input: AffiliateRelationUpdateRecord) -> AppResult<AffiliateRelationChangeRecord> {
        let mut state = self.state.lock().unwrap();
        let new_referrer_user_id = new_referrer_id(&state, input.referrer_aff_code.as_deref(), input.clear_referrer)?;
        let user = find_stored_user_mut(&mut state, &UserId(user_id.to_owned()))?;
        let old_referrer_user_id = user.user.referred_by_user_id.as_ref().map(|id| id.0.clone());
        user.user.referred_by_user_id = new_referrer_user_id.clone().map(UserId);
        user.user.referred_at = new_referrer_user_id.as_ref().map(|_| default_user_created_at());
        Ok(AffiliateRelationChangeRecord {
            id: "change-1".into(),
            user_id: user_id.into(),
            old_referrer_user_id,
            new_referrer_user_id,
            operator_user_id: input.operator_user_id,
            reason: input.reason,
            created_at: default_user_created_at().to_string(),
        })
    }

    async fn page_admin_affiliate_relation_changes(
        &self,
        request: PageSliceRequest,
        _query: AdminAffiliateRelationChangeQuery,
    ) -> AppResult<Page<AdminAffiliateRelationChangeItem>> {
        Ok(empty_page(request))
    }

    async fn page_admin_affiliate_commissions(
        &self,
        request: PageSliceRequest,
        _query: AdminAffiliateCommissionQuery,
    ) -> AppResult<Page<AdminAffiliateCommissionItem>> {
        Ok(empty_page(request))
    }

    async fn admin_affiliate_report(&self, query: AdminAffiliateReportQuery) -> AppResult<AdminAffiliateReportResponse> {
        Ok(AdminAffiliateReportResponse {
            daily_items: Vec::new(),
            referrer_items: Vec::new(),
            referrer_total: 0,
            page: query.page,
            page_size: query.page_size,
        })
    }

    async fn export_admin_affiliate_commissions(&self, _query: AdminAffiliateCommissionQuery) -> AppResult<Vec<AdminAffiliateCommissionItem>> {
        Ok(Vec::new())
    }

    async fn export_admin_affiliate_daily_report(&self, _query: AdminAffiliateReportQuery) -> AppResult<Vec<types::user::AdminAffiliateDailyReportItem>> {
        Ok(Vec::new())
    }

    async fn export_admin_affiliate_referrer_report(&self, _query: AdminAffiliateReportQuery) -> AppResult<Vec<types::user::AdminAffiliateReferrerReportItem>> {
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

impl PasswordHasher for TestPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        Ok(password_hash == format!("hashed:{password}"))
    }
}

impl SystemUserProvider for TestSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        Some(self.record.clone())
    }
}

impl StoredUser {
    fn auth_record(&self) -> UserAuthRecord {
        UserAuthRecord {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }

    pub(crate) fn referred_by(mut self, referrer_id: UserId) -> Self {
        self.user.referred_by_user_id = Some(referrer_id);
        self.user.referred_at = Some(default_user_created_at());
        self
    }

    pub(crate) fn regular_user(mut self) -> Self {
        self.user.role = constants::auth::DEFAULT_USER_ROLE.into();
        self
    }
}

pub(crate) fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: VALID_PASSWORD.into(),
        email: format!("{}@example.com", username.trim()),
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

pub(crate) fn replace_user(username: &str, is_active: bool) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: Some(VALID_PASSWORD.into()),
        email: format!("{}@example.com", username.trim()),
        role: "admin".into(),
        group_codes: vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()],
        is_active,
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_mode: USER_QUOTA_MODE_WALLET.into(),
    }
}

pub(crate) fn stored_user(id: u64, username: &str, password_hash: &str) -> StoredUser {
    StoredUser {
        user: User {
            id: user_id(id),
            username: username.into(),
            email: format!("{username}@example.com"),
            role: "admin".into(),
            group_codes: vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()],
            is_active: true,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
            email_verified: false,
            password_set: true,
            system: false,
            rate_limit_rpm: None,
            quota_mode: USER_QUOTA_MODE_WALLET.into(),
            affiliate_code: affiliate_code(&user_id(id)),
            referred_by_user_id: None,
            referred_at: None,
            created_at: default_user_created_at(),
            last_login_at: None,
        },
        password_hash: Some(password_hash.into()),
    }
}

pub(crate) fn passwordless_stored_user(id: u64, username: &str) -> StoredUser {
    let mut user = stored_user(id, username, "");
    user.user.password_set = false;
    user.password_hash = None;
    user
}

pub(crate) fn system_user() -> TestSystemUserProvider {
    TestSystemUserProvider {
        record: SystemUserRecord {
            user: User {
                id: user_id(0),
                username: "admin".into(),
                email: "admin@example.com".into(),
                role: "admin".into(),
                group_codes: vec![constants::user_group::DEFAULT_USER_GROUP_CODE.into()],
                is_active: true,
                allowed_model_ids: Vec::new(),
                allowed_provider_ids: Vec::new(),
                auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                email_verified: true,
                password_set: true,
                system: true,
                rate_limit_rpm: None,
                quota_mode: USER_QUOTA_MODE_WALLET.into(),
                affiliate_code: affiliate_code(&user_id(0)),
                referred_by_user_id: None,
                referred_at: None,
                created_at: default_user_created_at(),
                last_login_at: None,
            },
            password_hash: format!("hashed:{VALID_PASSWORD}"),
        },
    }
}

fn next_user_id(state: &mut RepositoryState) -> UserId {
    state.next_id += 1;
    user_id(state.next_id)
}

fn find_stored_user_mut<'a>(state: &'a mut RepositoryState, id: &UserId) -> AppResult<&'a mut StoredUser> {
    state.users.iter_mut().find(|stored| stored.user.id == *id).ok_or(AppError::NotFound)
}

fn replace_stored_user(state: &mut RepositoryState, id: &UserId, record: &ReplaceUserRecord) -> AppResult<User> {
    let stored = find_stored_user_mut(state, id)?;
    stored.user = updated_user(id.clone(), &stored.user, record);
    if let Some(password_hash) = &record.password_hash {
        stored.password_hash = Some(password_hash.clone());
        stored.user.password_set = true;
    }
    Ok(stored.user.clone())
}

fn user_from_record(id: UserId, record: &ReplaceUserRecord, referred_by_user_id: Option<UserId>) -> User {
    let referred_at = referred_by_user_id.as_ref().map(|_| default_user_created_at());
    User {
        affiliate_code: affiliate_code(&id),
        id,
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
        referred_by_user_id,
        referred_at,
        created_at: default_user_created_at(),
        last_login_at: None,
    }
}

fn identity_from_input(id: String, input: UserIdentityInput) -> UserIdentity {
    UserIdentity {
        id,
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
    }
}

fn updated_user(id: UserId, current: &User, record: &ReplaceUserRecord) -> User {
    User {
        email_verified: record.email_verified.unwrap_or(current.email_verified),
        password_set: current.password_set || record.password_hash.is_some(),
        affiliate_code: current.affiliate_code.clone(),
        referred_by_user_id: current.referred_by_user_id.clone(),
        referred_at: current.referred_at.clone(),
        created_at: current.created_at.clone(),
        last_login_at: current.last_login_at.clone(),
        ..user_from_record(id, record, current.referred_by_user_id.clone())
    }
}

pub(crate) fn user_id(id: u64) -> UserId {
    UserId(format!("018f0000-0000-7000-8000-{id:012}"))
}

pub(crate) fn affiliate_code(id: &UserId) -> String {
    id.0.chars().filter(|ch| *ch != '-').collect()
}

fn referrer_user_id(state: &RepositoryState, referrer_aff_code: Option<&str>) -> AppResult<Option<UserId>> {
    let Some(code) = referrer_aff_code else {
        return Ok(None);
    };
    state
        .users
        .iter()
        .find(|stored| stored.user.affiliate_code == code)
        .map(|stored| Some(stored.user.id.clone()))
        .ok_or_else(|| AppError::Conflict("referrer affiliate code does not exist".into()))
}

fn new_referrer_id(state: &RepositoryState, referrer_aff_code: Option<&str>, clear_referrer: bool) -> AppResult<Option<String>> {
    if clear_referrer {
        return Ok(None);
    }
    let Some(code) = referrer_aff_code else {
        return Ok(None);
    };
    let target = state
        .users
        .iter()
        .find(|stored| stored.user.affiliate_code == code)
        .ok_or_else(|| AppError::Conflict("referrer affiliate code does not exist".into()))?;
    if target.user.role != constants::auth::DEFAULT_USER_ROLE {
        return Err(AppError::Conflict("only regular users can be referrers".into()));
    }
    Ok(Some(target.user.id.0.clone()))
}

fn user_matches_filters(user: &User, filters: &UserListFilters) -> bool {
    if filters.is_active.is_some_and(|active| user.is_active != active) {
        return false;
    }
    if filters.role.as_ref().is_some_and(|role| user.role != *role) {
        return false;
    }
    if filters
        .group_code
        .as_ref()
        .is_some_and(|group_code| !user.group_codes.iter().any(|code| code == group_code))
    {
        return false;
    }
    filters.search.as_ref().is_none_or(|search| {
        user.username.contains(search) || user.email.contains(search) || user.role.contains(search) || user.group_codes.iter().any(|code| code.contains(search))
    })
}
