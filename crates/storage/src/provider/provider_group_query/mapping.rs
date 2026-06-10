use sea_orm::Set;
use types::{
    model::PatchField,
    provider::{ProviderGroup, ProviderGroupMember, ProviderKeyGroup, ProviderKeyGroupMember},
};

use super::super::{
    ProviderGroupRecordInput, ProviderGroupRecordPatch, ProviderKeyGroupRecordInput, ProviderKeyGroupRecordPatch,
    record::{ProviderGroupRecord, ProviderKeyGroupRecord, provider_groups, provider_key_groups},
};

pub fn provider_group_active_model(id: String, input: ProviderGroupRecordInput) -> provider_groups::ActiveModel {
    let now = time::OffsetDateTime::now_utc();
    provider_groups::ActiveModel {
        id: Set(id),
        name: Set(input.name),
        description: Set(input.description),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

pub fn provider_key_group_active_model(id: String, input: ProviderKeyGroupRecordInput) -> provider_key_groups::ActiveModel {
    let now = time::OffsetDateTime::now_utc();
    provider_key_groups::ActiveModel {
        id: Set(id),
        name: Set(input.name),
        description: Set(input.description),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

pub fn apply_provider_group_patch(active: &mut provider_groups::ActiveModel, input: ProviderGroupRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    apply_description_patch(&mut active.description, input.description);
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
}

pub fn apply_provider_key_group_patch(active: &mut provider_key_groups::ActiveModel, input: ProviderKeyGroupRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    apply_description_patch(&mut active.description, input.description);
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
}

pub fn provider_group_response(record: ProviderGroupRecord, provider_members: Vec<ProviderGroupMember>) -> ProviderGroup {
    ProviderGroup {
        id: record.id,
        name: record.name,
        description: record.description,
        sort_order: record.sort_order,
        provider_members,
        created_at: format_timestamp(record.created_at),
        updated_at: format_timestamp(record.updated_at),
    }
}

pub fn provider_key_group_response(record: ProviderKeyGroupRecord, provider_key_members: Vec<ProviderKeyGroupMember>) -> ProviderKeyGroup {
    ProviderKeyGroup {
        id: record.id,
        name: record.name,
        description: record.description,
        sort_order: record.sort_order,
        provider_key_members,
        created_at: format_timestamp(record.created_at),
        updated_at: format_timestamp(record.updated_at),
    }
}

pub fn unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<std::collections::BTreeSet<_>>().into_iter().collect()
}

fn apply_description_patch(target: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<String>) {
    match patch {
        PatchField::Value(value) => *target = Set(Some(value)),
        PatchField::Null => *target = Set(None),
        PatchField::Missing => {}
    }
}

fn format_timestamp(value: sea_orm::entity::prelude::TimeDateTimeWithTimeZone) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .expect("provider group timestamp must format as RFC3339")
}
