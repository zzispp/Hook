#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{billing_group_models, billing_groups};

pub type BillingGroupRecord = billing_groups::Model;
pub type BillingGroupModelRecord = billing_group_models::Model;
