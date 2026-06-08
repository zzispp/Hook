#[path = "entities/mod.rs"]
pub mod entities;

pub use crate::provider::record::{billing_group_provider_groups, billing_group_provider_key_groups};
pub use entities::{billing_group_models, billing_group_user_groups, billing_groups};

pub type BillingGroupRecord = billing_groups::Model;
pub type BillingGroupModelRecord = billing_group_models::Model;
pub type BillingGroupProviderGroupRecord = billing_group_provider_groups::Model;
pub type BillingGroupProviderKeyGroupRecord = billing_group_provider_key_groups::Model;
pub type BillingGroupUserGroupRecord = billing_group_user_groups::Model;
