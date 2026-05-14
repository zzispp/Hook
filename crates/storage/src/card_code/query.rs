use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use types::card_code::{CARD_CODE_STATUS_ACTIVE, CARD_CODE_STATUS_EXPIRED, CardCodeListFilters, CardCodeTypeListFilters};

use crate::card_code::{card_code_records, card_code_type_records};

pub(super) fn filtered_types(filters: CardCodeTypeListFilters) -> sea_orm::Select<card_code_type_records::Entity> {
    let mut query = card_code_type_records::Entity::find();
    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(card_code_type_records::Column::Status.eq(status));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(type_search_condition(&search)),
        _ => query,
    }
}

pub(super) fn filtered_codes(filters: CardCodeListFilters) -> sea_orm::Select<card_code_records::Entity> {
    let mut query = card_code_records::Entity::find();
    if let Some(type_id) = filters.type_id.filter(|value| !value.is_empty()) {
        query = query.filter(card_code_records::Column::TypeId.eq(type_id));
    }
    query = apply_status_filter(query, filters.status);
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(code_search_condition(&search)),
        _ => query,
    }
}

pub(super) fn not_expired_condition(now: time::OffsetDateTime) -> Condition {
    Condition::any()
        .add(card_code_records::Column::ExpiresAt.is_null())
        .add(card_code_records::Column::ExpiresAt.gt(now))
}

fn apply_status_filter(query: sea_orm::Select<card_code_records::Entity>, status: Option<String>) -> sea_orm::Select<card_code_records::Entity> {
    let now = time::OffsetDateTime::now_utc();
    match status.as_deref().filter(|value| !value.is_empty()) {
        Some(CARD_CODE_STATUS_ACTIVE) => active_codes(query, now),
        Some(CARD_CODE_STATUS_EXPIRED) => expired_codes(query, now),
        Some(value) => query.filter(card_code_records::Column::Status.eq(value)),
        None => query,
    }
}

fn active_codes(query: sea_orm::Select<card_code_records::Entity>, now: time::OffsetDateTime) -> sea_orm::Select<card_code_records::Entity> {
    query
        .filter(card_code_records::Column::Status.eq(CARD_CODE_STATUS_ACTIVE))
        .filter(not_expired_condition(now))
}

fn expired_codes(query: sea_orm::Select<card_code_records::Entity>, now: time::OffsetDateTime) -> sea_orm::Select<card_code_records::Entity> {
    query
        .filter(card_code_records::Column::Status.eq(CARD_CODE_STATUS_ACTIVE))
        .filter(card_code_records::Column::ExpiresAt.is_not_null())
        .filter(card_code_records::Column::ExpiresAt.lte(now))
}

fn type_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(card_code_type_records::Column::Name.contains(search))
        .add(card_code_type_records::Column::Remark.contains(search))
}

fn code_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(card_code_records::Column::Code.contains(search))
        .add(card_code_records::Column::BatchNo.contains(search))
        .add(card_code_records::Column::TypeName.contains(search))
        .add(card_code_records::Column::Remark.contains(search))
        .add(card_code_records::Column::CreatedByUsername.contains(search))
        .add(card_code_records::Column::CreatedIp.contains(search))
        .add(card_code_records::Column::UsedByUsername.contains(search))
        .add(card_code_records::Column::UsedIp.contains(search))
}
