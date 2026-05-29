use constants::pagination::MIN_PAGE_NUMBER;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId, UserListFilters},
};

use crate::application::{AppError, AppResult, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserRepository};

pub(super) fn reject_conflicting_system_user<S: SystemUserProvider>(system_users: &S, username: &str, email: &str) -> AppResult<()> {
    let Some(system) = system_users.system_user().map(|record| record.user) else {
        return Ok(());
    };
    reject_conflicting_field(username == system.username, "username")?;
    reject_conflicting_field(email == system.email, "email")
}

pub(super) fn reject_system_user_id<S: SystemUserProvider>(system_users: &S, id: &UserId) -> AppResult<()> {
    if system_user_by_id(system_users, id).is_some() {
        return Err(AppError::Conflict("system user cannot be changed".into()));
    }
    Ok(())
}

pub(super) fn system_user_by_id<S: SystemUserProvider>(system_users: &S, id: &UserId) -> Option<SystemUserRecord> {
    system_users.system_user().filter(|system_user| system_user.user.id == *id)
}

pub(super) async fn find_auth_by_identifier<R, S>(repository: &R, system_users: &S, identifier: &str) -> AppResult<Option<UserAuthRecord>>
where
    R: UserRepository,
    S: SystemUserProvider,
{
    if let Some(found) = system_auth_by_identifier(system_users, identifier) {
        return Ok(Some(found));
    }
    if let Some(found) = repository.find_auth_by_username(identifier).await? {
        return Ok(Some(found));
    }
    repository.find_auth_by_email(identifier).await
}

pub(super) async fn list_with_system_user<R: UserRepository>(
    repository: &R,
    page: PageRequest,
    filters: UserListFilters,
    system_user: User,
) -> AppResult<Page<User>> {
    let include_system = user_matches_filters(&system_user, &filters);
    let mut users = repository.list_slice(system_user_slice(page, include_system), filters).await?;
    if include_system && page.page == MIN_PAGE_NUMBER {
        users.items.insert(0, system_user);
    }
    if include_system {
        users.total += 1;
    }
    Ok(users)
}

fn system_auth_by_identifier<S: SystemUserProvider>(system_users: &S, identifier: &str) -> Option<UserAuthRecord> {
    let system_user = system_users.system_user()?;
    let user = system_user.user;
    if identifier != user.username && identifier != user.email {
        return None;
    }
    Some(UserAuthRecord {
        user,
        password_hash: Some(system_user.password_hash),
    })
}

fn system_user_slice(page: PageRequest, include_system: bool) -> PageSliceRequest {
    if include_system && page.page == MIN_PAGE_NUMBER {
        return PageSliceRequest {
            offset: 0,
            limit: page.page_size.saturating_sub(1),
            page: page.page,
            page_size: page.page_size,
        };
    }
    let system_offset = if include_system { MIN_PAGE_NUMBER } else { 0 };
    PageSliceRequest {
        offset: (page.page - MIN_PAGE_NUMBER) * page.page_size - system_offset,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

fn user_matches_filters(user: &User, filters: &UserListFilters) -> bool {
    if filters.is_active.is_some_and(|active| user.is_active != active) {
        return false;
    }
    if filters.role.as_ref().is_some_and(|role| user.role != *role) {
        return false;
    }
    if filters.group_code.as_ref().is_some_and(|group_code| user.group_code != *group_code) {
        return false;
    }
    filters.search.as_ref().is_none_or(|search| system_user_matches_search(user, search))
}

fn system_user_matches_search(user: &User, search: &str) -> bool {
    user.username.contains(search) || user.email.contains(search) || user.role.contains(search) || user.group_code.contains(search)
}

fn reject_conflicting_field(conflicting: bool, field: &str) -> AppResult<()> {
    if conflicting {
        return Err(AppError::Conflict(format!("{field} already exists")));
    }
    Ok(())
}
