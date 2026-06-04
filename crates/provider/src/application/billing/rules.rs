use rust_decimal::Decimal;
use serde_json::{Value, json};
use types::model::{PricingTier, TieredPricingConfig};

use super::{CACHE_CREATION_1H_TTL_MINUTES, CACHE_CREATION_5M_TTL_MINUTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BillingRuleScope {
    Model,
    Global,
    Default,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingRule {
    pub id: String,
    pub name: String,
    pub task_type: String,
    pub expression: String,
    pub variables: Value,
    pub dimension_mappings: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingRuleLookup {
    pub rule: BillingRule,
    pub scope: BillingRuleScope,
    pub effective_task_type: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltinRuleInput {
    pub global_model_name: String,
    pub task_type: String,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: TieredPricingConfig,
}

pub fn effective_rule_task_type(task_type: &str) -> String {
    if task_type.eq_ignore_ascii_case("cli") {
        "chat".into()
    } else {
        task_type.to_ascii_lowercase()
    }
}

pub fn universal_rule(input: BuiltinRuleInput) -> BillingRule {
    let prices = DefaultRulePrices::from_pricing(&input.tiered_pricing);
    let mappings = default_rule_mappings(&input.tiered_pricing);

    BillingRule {
        id: "__default__".into(),
        name: format!("Default rule for {}", input.global_model_name),
        task_type: effective_rule_task_type(&input.task_type),
        expression: default_expression().into(),
        variables: json!({
            "input_price_per_1m": prices.input,
            "output_price_per_1m": prices.output,
            "cache_creation_price_per_1m": prices.cache_creation,
            "cache_creation_ephemeral_5m_price_per_1m": prices.cache_creation_5m,
            "cache_creation_ephemeral_1h_price_per_1m": prices.cache_creation_1h,
            "cache_read_price_per_1m": prices.cache_read,
            "price_per_request": input.price_per_request.unwrap_or(Decimal::ZERO)
        }),
        dimension_mappings: mappings,
    }
}

#[derive(Clone, Copy, Debug)]
struct DefaultRulePrices {
    input: Decimal,
    output: Decimal,
    cache_creation: Decimal,
    cache_creation_5m: Decimal,
    cache_creation_1h: Decimal,
    cache_read: Decimal,
}

impl DefaultRulePrices {
    fn from_pricing(pricing: &TieredPricingConfig) -> Self {
        let first = pricing.tiers.first();
        let input = first.map(|tier| tier.input_price_per_1m).unwrap_or(Decimal::ZERO);
        let output = first.map(|tier| tier.output_price_per_1m).unwrap_or(Decimal::ZERO);
        let cache_creation = first
            .and_then(|tier| tier.cache_creation_price_per_1m)
            .unwrap_or(default_cache_creation_price(input));
        let cache_creation_5m = first
            .and_then(|tier| cache_creation_ttl_price(tier, CACHE_CREATION_5M_TTL_MINUTES))
            .unwrap_or(cache_creation);
        let cache_creation_1h = first
            .and_then(|tier| cache_creation_ttl_price(tier, CACHE_CREATION_1H_TTL_MINUTES))
            .unwrap_or(cache_creation);
        let cache_read = first.and_then(cache_read_price).unwrap_or(default_cache_read_price(input));
        Self {
            input,
            output,
            cache_creation,
            cache_creation_5m,
            cache_creation_1h,
            cache_read,
        }
    }
}

fn default_rule_mappings(pricing: &TieredPricingConfig) -> Value {
    let mut mappings = default_dimension_mappings();
    if pricing.tiers.is_empty() {
        return mappings;
    }
    mappings["input_price_per_1m"] = tiered_mapping("input_price_per_1m", pricing, None);
    mappings["output_price_per_1m"] = tiered_mapping("output_price_per_1m", pricing, None);
    mappings["cache_creation_price_per_1m"] = tiered_mapping("cache_creation_price_per_1m", pricing, Some("cache_creation_price_per_1m"));
    mappings["cache_creation_ephemeral_5m_price_per_1m"] =
        tiered_mapping("cache_creation_ephemeral_5m_price_per_1m", pricing, Some("cache_creation_price_per_1m"));
    mappings["cache_creation_ephemeral_1h_price_per_1m"] =
        tiered_mapping("cache_creation_ephemeral_1h_price_per_1m", pricing, Some("cache_creation_price_per_1m"));
    mappings["cache_read_price_per_1m"] = tiered_mapping("cache_read_price_per_1m", pricing, Some("cache_read_price_per_1m"));
    mappings
}

fn default_dimension_mappings() -> Value {
    json!({
        "input_tokens": {"source": "dimension", "key": "input_tokens", "required": false, "allow_zero": true, "default": 0},
        "output_tokens": {"source": "dimension", "key": "output_tokens", "required": false, "allow_zero": true, "default": 0},
        "cache_creation_tokens": {"source": "dimension", "key": "cache_creation_tokens", "required": false, "allow_zero": true, "default": 0},
        "cache_creation_5m_input_tokens": {"source": "dimension", "key": "cache_creation_5m_input_tokens", "required": false, "allow_zero": true, "default": 0},
        "cache_creation_1h_input_tokens": {"source": "dimension", "key": "cache_creation_1h_input_tokens", "required": false, "allow_zero": true, "default": 0},
        "cache_read_tokens": {"source": "dimension", "key": "cache_read_tokens", "required": false, "allow_zero": true, "default": 0},
        "request_count": {"source": "dimension", "key": "request_count", "required": false, "allow_zero": true, "default": 1},
        "duration_seconds": {"source": "dimension", "key": "duration_seconds", "required": false, "allow_zero": true, "default": 0},
        "video_price_per_second": {"source": "constant", "default": 0},
        "cache_creation_uncategorized_tokens": {"source": "dimension", "key": "cache_creation_uncategorized_tokens", "required": false, "allow_zero": true, "default": 0},
        "input_cost": {"source": "computed", "expression": "input_tokens * input_price_per_1m / 1000000", "required": false, "default": 0},
        "output_cost": {"source": "computed", "expression": "output_tokens * output_price_per_1m / 1000000", "required": false, "default": 0},
        "cache_creation_uncategorized_cost": {
            "source": "computed",
            "expression": "cache_creation_uncategorized_tokens * cache_creation_price_per_1m / 1000000",
            "required": false,
            "default": 0
        },
        "cache_creation_ephemeral_5m_cost": {
            "source": "computed",
            "expression": "cache_creation_5m_input_tokens * cache_creation_ephemeral_5m_price_per_1m / 1000000",
            "required": false,
            "default": 0
        },
        "cache_creation_ephemeral_1h_cost": {
            "source": "computed",
            "expression": "cache_creation_1h_input_tokens * cache_creation_ephemeral_1h_price_per_1m / 1000000",
            "required": false,
            "default": 0
        },
        "cache_read_cost": {"source": "computed", "expression": "cache_read_tokens * cache_read_price_per_1m / 1000000", "required": false, "default": 0},
        "request_cost": {"source": "computed", "expression": "request_count * price_per_request", "required": false, "default": 0},
        "video_cost": {"source": "computed", "expression": "duration_seconds * video_price_per_second", "required": false, "default": 0}
    })
}

fn default_expression() -> &'static str {
    "input_cost + output_cost + cache_creation_uncategorized_cost + cache_creation_ephemeral_5m_cost + cache_creation_ephemeral_1h_cost + cache_read_cost + request_cost + video_cost"
}

fn tiered_mapping(key: &str, pricing: &TieredPricingConfig, ttl_value_key: Option<&str>) -> Value {
    let tiers = pricing.tiers.iter().map(|tier| tier_json(key, tier, ttl_value_key)).collect::<Vec<_>>();
    let mut mapping = json!({
        "source": "tiered",
        "tier_key": "total_input_context",
        "allow_zero": true,
        "tiers": tiers,
        "default": 0
    });
    if let Some(ttl_value_key) = ttl_value_key {
        mapping["ttl_key"] = Value::String(ttl_key(key).into());
        mapping["ttl_value_key"] = Value::String(ttl_value_key.into());
    }
    mapping
}

fn tier_json(key: &str, tier: &PricingTier, ttl_value_key: Option<&str>) -> Value {
    let value = match key {
        "input_price_per_1m" => tier.input_price_per_1m,
        "output_price_per_1m" => tier.output_price_per_1m,
        "cache_creation_price_per_1m" => tier.cache_creation_price_per_1m.unwrap_or(tier.input_price_per_1m * Decimal::new(125, 2)),
        "cache_creation_ephemeral_5m_price_per_1m" => cache_creation_ttl_price(tier, CACHE_CREATION_5M_TTL_MINUTES).unwrap_or(cache_creation_price(tier)),
        "cache_creation_ephemeral_1h_price_per_1m" => cache_creation_ttl_price(tier, CACHE_CREATION_1H_TTL_MINUTES).unwrap_or(cache_creation_price(tier)),
        "cache_read_price_per_1m" => cache_read_price(tier).unwrap_or(tier.input_price_per_1m * Decimal::new(1, 1)),
        _ => Decimal::ZERO,
    };
    let mut item = json!({"up_to": tier.up_to, "value": value});
    if ttl_value_key.is_some()
        && let Some(ttl) = &tier.cache_ttl_pricing
    {
        item["cache_ttl_pricing"] = json!(ttl);
    }
    item
}

fn ttl_key(key: &str) -> &'static str {
    match key {
        "cache_creation_ephemeral_5m_price_per_1m" => "cache_creation_ephemeral_5m_ttl_minutes",
        "cache_creation_ephemeral_1h_price_per_1m" => "cache_creation_ephemeral_1h_ttl_minutes",
        _ => "cache_ttl_minutes",
    }
}

fn cache_creation_price(tier: &PricingTier) -> Decimal {
    tier.cache_creation_price_per_1m
        .unwrap_or(default_cache_creation_price(tier.input_price_per_1m))
}

fn default_cache_creation_price(input_price: Decimal) -> Decimal {
    input_price * Decimal::new(125, 2)
}

fn default_cache_read_price(input_price: Decimal) -> Decimal {
    input_price * Decimal::new(1, 1)
}

fn cache_creation_ttl_price(tier: &PricingTier, ttl_minutes: i64) -> Option<Decimal> {
    let target = u64::try_from(ttl_minutes).ok()?;
    tier.cache_ttl_pricing
        .as_ref()?
        .iter()
        .find(|item| item.ttl_minutes == target)
        .map(|item| item.cache_creation_price_per_1m)
}

fn cache_read_price(tier: &PricingTier) -> Option<Decimal> {
    tier.cache_read_price_per_1m.or_else(|| {
        tier.cache_ttl_pricing
            .as_ref()?
            .iter()
            .find(|item| item.ttl_minutes == 5)
            .and_then(|item| item.cache_read_price_per_1m)
    })
}
