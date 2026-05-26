use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, Select};
use types::user::UserListFilters;

use super::{UserColumn, UserEntity as Users};

pub(super) fn active_users() -> Select<Users> {
    Users::find().filter(UserColumn::IsDeleted.eq(false))
}

pub(super) fn filtered_users(filters: UserListFilters) -> Select<Users> {
    let mut query = active_users();
    if let Some(is_active) = filters.is_active {
        query = query.filter(UserColumn::IsActive.eq(is_active));
    }
    if let Some(role) = filters.role {
        query = query.filter(UserColumn::Role.eq(role));
    }
    if let Some(group_code) = filters.group_code {
        query = query.filter(UserColumn::GroupCode.eq(group_code));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(user_search_condition(&search)),
        _ => query,
    }
}

fn user_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(UserColumn::Username.contains(search))
        .add(UserColumn::Email.contains(search))
        .add(UserColumn::Role.contains(search))
        .add(UserColumn::GroupCode.contains(search))
}
