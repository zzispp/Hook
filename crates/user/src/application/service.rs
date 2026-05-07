use async_trait::async_trait;
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};

use crate::application::{AppError, AppResult, PasswordHasher, ReplaceUserRecord, UserAuthRecord, UserRepository, UserUseCase};
use types::user::{Credentials, NewUser, Page, PageRequest, ReplaceUser, User, UserId};

pub struct UserService<R, H> {
    repository: R,
    password_hasher: H,
}

struct UserRecordInput {
    username: String,
    password: String,
    email: String,
    role: String,
    status: String,
}

impl<R, H> UserService<R, H>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub const fn new(repository: R, password_hasher: H) -> Self {
        Self { repository, password_hasher }
    }

    async fn create_unique_user(&self, input: NewUser) -> AppResult<User> {
        validate_new_user(&input)?;
        self.ensure_unique_user(&input.username, &input.email, None).await?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, username: &str, email: &str, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(username).await? {
            reject_conflicting_user(found.user.id, current_id, "username")?;
        }

        if let Some(found) = self.repository.find_by_email(email).await? {
            reject_conflicting_user(found.id, current_id, "email")?;
        }

        Ok(())
    }

    fn new_user_record(&self, input: NewUser) -> AppResult<ReplaceUserRecord> {
        self.to_record(UserRecordInput::from(input))
    }

    fn replace_user_record(&self, input: ReplaceUser) -> AppResult<ReplaceUserRecord> {
        self.to_record(UserRecordInput::from(input))
    }

    fn to_record(&self, input: UserRecordInput) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: self.password_hasher.hash(&input.password)?,
            email: input.email,
            role: input.role,
            status: input.status,
        })
    }
}

#[async_trait]
impl<R, H> UserUseCase for UserService<R, H>
where
    R: UserRepository,
    H: PasswordHasher,
{
    async fn sign_up(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn sign_in(&self, input: Credentials) -> AppResult<User> {
        validate_credentials(&input)?;
        let found = self.repository.find_auth_by_username(&input.username).await?.ok_or(AppError::Unauthorized)?;
        verify_password(&self.password_hasher, &input.password, &found)?;
        Ok(found.user)
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        validate_replace_user(&input)?;
        ensure_user_exists(self.repository.find_by_id(id).await?)?;
        self.ensure_unique_user(&input.username, &input.email, Some(id)).await?;
        self.repository.replace(id, self.replace_user_record(input)?).await
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        self.repository.delete(id).await
    }

    async fn list_users(&self, page: PageRequest) -> AppResult<Page<User>> {
        validate_page(page)?;
        self.repository.list(page).await
    }
}

impl From<NewUser> for UserRecordInput {
    fn from(value: NewUser) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            status: value.status,
        }
    }
}

impl From<ReplaceUser> for UserRecordInput {
    fn from(value: ReplaceUser) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            status: value.status,
        }
    }
}

fn ensure_user_exists(user: Option<User>) -> AppResult<()> {
    match user {
        Some(_) => Ok(()),
        None => Err(AppError::NotFound),
    }
}

fn reject_conflicting_user(id: UserId, current_id: Option<UserId>, field: &str) -> AppResult<()> {
    if current_id == Some(id) {
        return Ok(());
    }

    Err(AppError::Conflict(format!("{field} already exists")))
}

fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    if hasher.verify(password, &found.password_hash)? {
        return Ok(());
    }

    Err(AppError::Unauthorized)
}

fn validate_credentials(input: &Credentials) -> AppResult<()> {
    reject_blank("username", &input.username)?;
    reject_blank("password", &input.password)
}

fn validate_new_user(input: &NewUser) -> AppResult<()> {
    reject_blank("username", &input.username)?;
    reject_blank("password", &input.password)?;
    reject_blank("email", &input.email)?;
    reject_blank("role", &input.role)?;
    reject_blank("status", &input.status)
}

fn validate_replace_user(input: &ReplaceUser) -> AppResult<()> {
    reject_blank("username", &input.username)?;
    reject_blank("password", &input.password)?;
    reject_blank("email", &input.email)?;
    reject_blank("role", &input.role)?;
    reject_blank("status", &input.status)
}

fn validate_page(page: PageRequest) -> AppResult<()> {
    if page.page < MIN_PAGE_NUMBER {
        return Err(AppError::InvalidInput("page must be greater than 0".into()));
    }

    if page.page_size < MIN_PAGE_SIZE {
        return Err(AppError::InvalidInput("page_size must be greater than 0".into()));
    }

    if page.page_size > MAX_PAGE_SIZE {
        return Err(AppError::InvalidInput(format!("page_size must be less than or equal to {MAX_PAGE_SIZE}")));
    }

    Ok(())
}

fn reject_blank(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be blank")));
    }

    Ok(())
}

#[cfg(test)]
mod tests;
