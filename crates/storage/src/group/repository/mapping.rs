use sea_orm::Set;
use types::{group::BillingGroup, model::PatchField};

use crate::group::{
    BillingGroupRecordInput, BillingGroupRecordPatch,
    record::{BillingGroupRecord, billing_groups},
};

pub fn group_active_model(id: String, input: BillingGroupRecordInput) -> billing_groups::ActiveModel {
    let now = time::OffsetDateTime::now_utc();
    billing_groups::ActiveModel {
        id: Set(id),
        code: Set(input.code),
        name: Set(input.name),
        description: Set(input.description),
        billing_multiplier: Set(input.billing_multiplier),
        is_active: Set(input.is_active),
        is_system: Set(input.is_system),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

pub fn domain_group(
    record: BillingGroupRecord,
    allowed_model_ids: Vec<String>,
    allowed_provider_group_ids: Vec<String>,
    allowed_provider_key_group_ids: Vec<String>,
    visible_user_group_codes: Vec<String>,
) -> BillingGroup {
    BillingGroup {
        allowed_model_ids,
        allowed_provider_group_ids,
        allowed_provider_key_group_ids,
        visible_user_group_codes,
        ..BillingGroup::from(record)
    }
}

pub fn apply_group_patch(active: &mut billing_groups::ActiveModel, input: BillingGroupRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    apply_description_patch(&mut active.description, input.description);
    if let Some(multiplier) = input.billing_multiplier {
        active.billing_multiplier = Set(multiplier);
    }
    if let Some(is_active) = input.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
}

fn apply_description_patch(target: &mut sea_orm::ActiveValue<Option<String>>, patch: PatchField<String>) {
    match patch {
        PatchField::Value(value) => *target = Set(Some(value)),
        PatchField::Null => *target = Set(None),
        PatchField::Missing => {}
    }
}
