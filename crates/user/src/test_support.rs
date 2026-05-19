use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{NewUser, ReplaceUser, USER_QUOTA_MODE_WALLET, User, UserId, UserListFilters, default_user_created_at},
};

use crate::application::{
    AppError, AppResult, PasswordHasher, PasswordResetRecord, PasswordResetRepository, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord,
    UserRepository,
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
}

#[derive(Clone)]
pub(crate) struct StoredUser {
    user: User,
    password_hash: String,
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
}

#[async_trait]
impl UserRepository for MemoryUserRepository {
    async fn create(&self, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let id = next_user_id(&mut state);
        let user = user_from_record(id, &record);
        state.users.push(StoredUser {
            user: user.clone(),
            password_hash: required_password_hash(&record)?,
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
        stored.password_hash = password_hash.to_owned();
        let user = stored.user.clone();
        state.reset_tokens[index].consumed_at = Some(now);
        Ok(Some(user))
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
}

pub(crate) fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: VALID_PASSWORD.into(),
        email: format!("{}@example.com", username.trim()),
        role: "admin".into(),
        is_active: true,
        allowed_model_ids: Vec::new(),
        allowed_provider_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_mode: USER_QUOTA_MODE_WALLET.into(),
    }
}

pub(crate) fn replace_user(username: &str, is_active: bool) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: Some(VALID_PASSWORD.into()),
        email: format!("{}@example.com", username.trim()),
        role: "admin".into(),
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
            is_active: true,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
            email_verified: false,
            system: false,
            rate_limit_rpm: None,
            quota_mode: USER_QUOTA_MODE_WALLET.into(),
            created_at: default_user_created_at(),
            last_login_at: None,
        },
        password_hash: password_hash.into(),
    }
}

pub(crate) fn system_user() -> TestSystemUserProvider {
    TestSystemUserProvider {
        record: SystemUserRecord {
            user: User {
                id: user_id(0),
                username: "admin".into(),
                email: "admin@example.com".into(),
                role: "admin".into(),
                is_active: true,
                allowed_model_ids: Vec::new(),
                allowed_provider_ids: Vec::new(),
                auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                email_verified: true,
                system: true,
                rate_limit_rpm: None,
                quota_mode: USER_QUOTA_MODE_WALLET.into(),
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
    stored.password_hash = required_password_hash(record)?;
    Ok(stored.user.clone())
}

fn user_from_record(id: UserId, record: &ReplaceUserRecord) -> User {
    User {
        id,
        username: record.username.clone(),
        email: record.email.clone(),
        role: record.role.clone(),
        is_active: record.is_active,
        allowed_model_ids: record.allowed_model_ids.clone(),
        allowed_provider_ids: record.allowed_provider_ids.clone(),
        auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
        email_verified: record.email_verified.unwrap_or(false),
        system: false,
        rate_limit_rpm: record.rate_limit_rpm,
        quota_mode: record.quota_mode.clone(),
        created_at: default_user_created_at(),
        last_login_at: None,
    }
}

fn updated_user(id: UserId, current: &User, record: &ReplaceUserRecord) -> User {
    User {
        email_verified: record.email_verified.unwrap_or(current.email_verified),
        created_at: current.created_at.clone(),
        last_login_at: current.last_login_at.clone(),
        ..user_from_record(id, record)
    }
}

fn required_password_hash(record: &ReplaceUserRecord) -> AppResult<String> {
    record
        .password_hash
        .clone()
        .ok_or_else(|| AppError::InvalidInput("password_hash is required".into()))
}

pub(crate) fn user_id(id: u64) -> UserId {
    UserId(format!("018f0000-0000-7000-8000-{id:012}"))
}

fn user_matches_filters(user: &User, filters: &UserListFilters) -> bool {
    if filters.is_active.is_some_and(|active| user.is_active != active) {
        return false;
    }
    if filters.role.as_ref().is_some_and(|role| user.role != *role) {
        return false;
    }
    filters
        .search
        .as_ref()
        .is_none_or(|search| user.username.contains(search) || user.email.contains(search) || user.role.contains(search))
}
