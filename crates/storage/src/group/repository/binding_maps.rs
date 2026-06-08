use std::collections::BTreeMap;

use crate::group::record::{BillingGroupModelRecord, BillingGroupProviderGroupRecord, BillingGroupProviderKeyGroupRecord, BillingGroupUserGroupRecord};

pub fn model_bindings_by_group(records: Vec<BillingGroupModelRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.group_code).or_default().push(record.global_model_id);
    }
    bindings
}

pub fn provider_group_bindings_by_group(records: Vec<BillingGroupProviderGroupRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.group_code).or_default().push(record.provider_group_id);
    }
    bindings
}

pub fn provider_key_group_bindings_by_group(records: Vec<BillingGroupProviderKeyGroupRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.group_code).or_default().push(record.provider_key_group_id);
    }
    bindings
}

pub fn user_group_bindings_by_group(records: Vec<BillingGroupUserGroupRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.billing_group_code).or_default().push(record.user_group_code);
    }
    bindings
}

pub fn binding_model_ids(records: Vec<BillingGroupModelRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.global_model_id).collect()
}

pub fn binding_provider_group_ids(records: Vec<BillingGroupProviderGroupRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.provider_group_id).collect()
}

pub fn binding_provider_key_group_ids(records: Vec<BillingGroupProviderKeyGroupRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.provider_key_group_id).collect()
}

pub fn binding_user_group_codes(records: Vec<BillingGroupUserGroupRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.user_group_code).collect()
}
