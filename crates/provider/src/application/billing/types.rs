use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const BILLING_SNAPSHOT_SCHEMA_VERSION: &str = "2.0";
pub const BILLING_SCALE: u32 = 8;
pub const ACCOUNTING_CURRENCY: &str = currency::ACCOUNTING_CURRENCY;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingSnapshotStatus {
    Complete,
    Incomplete,
    NoRule,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BillingSnapshot {
    pub schema_version: String,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub scope: Option<String>,
    pub expression: Option<String>,
    pub resolved_dimensions: BTreeMap<String, serde_json::Value>,
    pub resolved_variables: BTreeMap<String, serde_json::Value>,
    pub cost_breakdown: BTreeMap<String, Decimal>,
    pub base_total_cost: Decimal,
    pub total_cost: Decimal,
    pub group_code: Option<String>,
    pub billing_multiplier: Decimal,
    pub tier_index: Option<usize>,
    pub tier_info: Option<serde_json::Value>,
    pub missing_required: Vec<String>,
    pub status: BillingSnapshotStatus,
    pub calculated_at: String,
    pub engine_version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CostResult {
    pub cost: Decimal,
    pub status: BillingSnapshotStatus,
    pub snapshot: BillingSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RequestBillingAmount {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub cache_creation_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub request_cost: Decimal,
    pub token_cost: Decimal,
    pub base_cost: Decimal,
    pub total_cost: Decimal,
    pub billing_multiplier: Decimal,
    pub input_price_per_1m: Option<Decimal>,
    pub output_price_per_1m: Option<Decimal>,
    pub cache_creation_price_per_1m: Option<Decimal>,
    pub cache_read_price_per_1m: Option<Decimal>,
    pub currency: String,
    pub snapshot: BillingSnapshot,
}

pub fn quantize(value: Decimal) -> Decimal {
    value.round_dp(BILLING_SCALE)
}
