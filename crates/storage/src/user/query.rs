use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, QueryTrait, Select};
use types::user::UserListFilters;

use super::{UserColumn, UserEntity as Users, user_group_memberships};

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
        query = query.filter(UserColumn::Id.in_subquery(user_ids_for_group_code(&group_code)));
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
        .add(UserColumn::Id.in_subquery(user_ids_for_group_search(search)))
}

fn user_ids_for_group_code(group_code: &str) -> sea_orm::sea_query::SelectStatement {
    user_group_memberships::Entity::find()
        .select_only()
        .column(user_group_memberships::Column::UserId)
        .filter(user_group_memberships::Column::UserGroupCode.eq(group_code))
        .into_query()
}

fn user_ids_for_group_search(search: &str) -> sea_orm::sea_query::SelectStatement {
    user_group_memberships::Entity::find()
        .select_only()
        .column(user_group_memberships::Column::UserId)
        .filter(user_group_memberships::Column::UserGroupCode.contains(search))
        .into_query()
}
