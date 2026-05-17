use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde_json::Value;

use super::{MappingResolution, TierMeta};
use crate::application::billing::formula::value::value_decimal;

pub(super) fn resolve_tiered(
    mapping: &serde_json::Map<String, Value>,
    dimensions: &BTreeMap<String, Value>,
    fallback: impl Fn(bool, Value) -> MappingResolution,
) -> MappingResolution {
    let required = mapping_bool(mapping, "required");
    let allow_zero = mapping_bool(mapping, "allow_zero");
    let default = mapping.get("default").cloned().unwrap_or(Value::from(0));
    let Some(tier_key) = mapping.get("tier_key").and_then(Value::as_str) else {
        return fallback(required, default);
    };
    let Some(raw) = dimensions.get(tier_key) else {
        return fallback(required, default);
    };
    let Ok(tier_value) = value_decimal(raw) else {
        return fallback(required, default);
    };
    if tier_value == Decimal::ZERO && !allow_zero {
        return fallback(required, default);
    }
    tiered_value(mapping, dimensions, tier_value, default)
}

fn tiered_value(mapping: &serde_json::Map<String, Value>, dimensions: &BTreeMap<String, Value>, tier_value: Decimal, default: Value) -> MappingResolution {
    let tiers = mapping.get("tiers").and_then(Value::as_array).cloned().unwrap_or_default();
    for (index, tier) in tiers.iter().enumerate() {
        let Some(tier_obj) = tier.as_object() else { continue };
        if tier_matches(tier_obj.get("up_to"), tier_value) {
            return tier_resolution(index, tier.clone(), mapping, tier_obj, &default, dimensions);
        }
    }
    if let Some((index, tier)) = tiers.iter().enumerate().next_back() {
        let tier_obj = tier.as_object().cloned().unwrap_or_default();
        return tier_resolution(index, tier.clone(), mapping, &tier_obj, &default, dimensions);
    }
    MappingResolution {
        value: default,
        missing_required: false,
        tier_meta: None,
    }
}

fn tier_resolution(
    index: usize,
    tier: Value,
    mapping: &serde_json::Map<String, Value>,
    tier_obj: &serde_json::Map<String, Value>,
    default: &Value,
    dimensions: &BTreeMap<String, Value>,
) -> MappingResolution {
    MappingResolution {
        value: tier_value_for_mapping(mapping, tier_obj, default, dimensions),
        missing_required: false,
        tier_meta: Some(TierMeta {
            tier_index: Some(index),
            tier_info: Some(tier),
        }),
    }
}

fn tier_matches(up_to: Option<&Value>, tier_value: Decimal) -> bool {
    match up_to {
        None | Some(Value::Null) => true,
        Some(value) => value_decimal(value).is_ok_and(|limit| tier_value <= limit),
    }
}

fn tier_value_for_mapping(
    mapping: &serde_json::Map<String, Value>,
    tier: &serde_json::Map<String, Value>,
    default: &Value,
    dimensions: &BTreeMap<String, Value>,
) -> Value {
    let base = tier.get("value").cloned().unwrap_or_else(|| default.clone());
    let Some(ttl_key) = mapping.get("ttl_key").and_then(Value::as_str) else {
        return base;
    };
    let Some(ttl_value_key) = mapping.get("ttl_value_key").and_then(Value::as_str) else {
        return base;
    };
    let Some(ttl_minutes) = dimensions.get(ttl_key).and_then(|value| value_decimal(value).ok()) else {
        return base;
    };
    resolve_ttl_price(tier, ttl_minutes, ttl_value_key).unwrap_or(base)
}

fn resolve_ttl_price(tier: &serde_json::Map<String, Value>, ttl_minutes: Decimal, value_key: &str) -> Option<Value> {
    let entries = tier.get("cache_ttl_pricing")?.as_array()?;
    let mut last = None;
    for entry in entries.iter().filter_map(Value::as_object) {
        last = entry.get(value_key).cloned();
        let Some(limit) = entry.get("ttl_minutes").and_then(|value| value_decimal(value).ok()) else {
            continue;
        };
        if ttl_minutes <= limit {
            return entry.get(value_key).cloned();
        }
    }
    last
}

fn mapping_bool(mapping: &serde_json::Map<String, Value>, key: &str) -> bool {
    mapping.get(key).and_then(Value::as_bool).unwrap_or(false)
}
