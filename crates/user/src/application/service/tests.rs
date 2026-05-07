use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use constants::pagination::MAX_PAGE_SIZE;

use crate::application::{AppError, AppResult, PasswordHasher, ReplaceUserRecord, UserAuthRecord, UserRepository, UserService, UserUseCase};
use types::user::{Credentials, NewUser, Page, PageRequest, ReplaceUser, User, UserId};

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
            username: "alice".into(),
            password: "bad-password".into(),
        })
        .await;

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

#[derive(Clone, Default)]
struct MemoryUserRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Default)]
struct RepositoryState {
    next_id: u64,
    users: Vec<StoredUser>,
    created: Vec<ReplaceUserRecord>,
    replaced: Vec<(UserId, ReplaceUserRecord)>,
    deleted: Vec<UserId>,
}

#[derive(Clone)]
struct StoredUser {
    user: User,
    password_hash: String,
}

#[derive(Clone)]
struct TestPasswordHasher;

impl MemoryUserRepository {
    fn with_user(user: StoredUser) -> Self {
        let repository = Self::default();
        repository.state.lock().unwrap().users.push(user);
        repository
    }

    fn created_records(&self) -> Vec<ReplaceUserRecord> {
        self.state.lock().unwrap().created.clone()
    }

    fn replaced_records(&self) -> Vec<(UserId, ReplaceUserRecord)> {
        self.state.lock().unwrap().replaced.clone()
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
            password_hash: record.password_hash.clone(),
        });
        state.created.push(record);
        Ok(user)
    }

    async fn replace(&self, id: UserId, record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        let user = replace_stored_user(&mut state, id, &record)?;
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

    async fn list(&self, page: PageRequest) -> AppResult<Page<User>> {
        let state = self.state.lock().unwrap();
        Ok(Page {
            items: state.users.iter().map(|stored| stored.user.clone()).collect(),
            total: state.users.len() as u64,
            page: page.page,
            page_size: page.page_size,
        })
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

impl StoredUser {
    fn auth_record(&self) -> UserAuthRecord {
        UserAuthRecord {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }
}

fn next_user_id(state: &mut RepositoryState) -> UserId {
    state.next_id += 1;
    UserId(state.next_id)
}

fn find_stored_user_mut(state: &mut RepositoryState, id: UserId) -> AppResult<&mut StoredUser> {
    state.users.iter_mut().find(|stored| stored.user.id == id).ok_or(AppError::NotFound)
}

fn replace_stored_user(state: &mut RepositoryState, id: UserId, record: &ReplaceUserRecord) -> AppResult<User> {
    let stored = find_stored_user_mut(state, id)?;
    stored.user = user_from_record(id, record);
    stored.password_hash = record.password_hash.clone();
    Ok(stored.user.clone())
}

fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: "secret".into(),
        email: format!("{username}@example.com"),
        role: "admin".into(),
        status: "enabled".into(),
    }
}

fn replace_user(username: &str, status: &str) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: "secret".into(),
        email: format!("{username}@example.com"),
        role: "admin".into(),
        status: status.into(),
    }
}

fn stored_user(id: u64, username: &str, password_hash: &str) -> StoredUser {
    StoredUser {
        user: User {
            id: UserId(id),
            username: username.into(),
            email: format!("{username}@example.com"),
            role: "admin".into(),
            status: "enabled".into(),
        },
        password_hash: password_hash.into(),
    }
}

fn user_from_record(id: UserId, record: &ReplaceUserRecord) -> User {
    User {
        id,
        username: record.username.clone(),
        email: record.email.clone(),
        role: record.role.clone(),
        status: record.status.clone(),
    }
}
