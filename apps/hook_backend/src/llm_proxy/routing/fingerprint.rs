use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use types::{model::TieredPricingConfig, provider::ProviderModelCost};

pub(crate) struct RouteFingerprintInput<'a> {
    pub(crate) provider_id: &'a str,
    pub(crate) key_id: &'a str,
    pub(crate) endpoint_id: &'a str,
    pub(crate) global_model_id: &'a str,
    pub(crate) provider_model_id: &'a str,
    pub(crate) effective_upstream_model_name: &'a str,
    pub(crate) effective_reasoning_effort: Option<&'a str>,
    pub(crate) client_api_format: &'a str,
    pub(crate) provider_api_format: &'a str,
    pub(crate) is_stream: bool,
    pub(crate) needs_conversion: bool,
}

pub(crate) struct PriceFingerprintInput<'a> {
    pub(crate) configured_cost: Option<&'a ProviderModelCost>,
    pub(crate) price_per_request: Option<Decimal>,
    pub(crate) tiered_pricing: &'a TieredPricingConfig,
    pub(crate) billing_multiplier: Decimal,
}

pub(crate) fn route_config_fingerprint(input: RouteFingerprintInput<'_>) -> String {
    fingerprint(&[
        ("provider", input.provider_id.to_owned()),
        ("key", input.key_id.to_owned()),
        ("endpoint", input.endpoint_id.to_owned()),
        ("global_model", input.global_model_id.to_owned()),
        ("provider_model", input.provider_model_id.to_owned()),
        ("effective_upstream_model", input.effective_upstream_model_name.to_owned()),
        (
            "effective_reasoning_effort",
            input.effective_reasoning_effort.map(str::to_owned).unwrap_or_else(|| "none".into()),
        ),
        ("client_format", input.client_api_format.to_owned()),
        ("provider_format", input.provider_api_format.to_owned()),
        ("stream", input.is_stream.to_string()),
        ("conversion", input.needs_conversion.to_string()),
    ])
}

pub(crate) fn price_config_fingerprint(input: PriceFingerprintInput<'_>) -> String {
    fingerprint(&[
        ("configured_cost", configured_cost(input.configured_cost)),
        ("price_per_request", optional_decimal(input.price_per_request)),
        (
            "tiered_pricing",
            serde_json::to_string(input.tiered_pricing).expect("tiered pricing config must serialize"),
        ),
        ("billing_multiplier", input.billing_multiplier.to_string()),
    ])
}

fn fingerprint(parts: &[(&str, String)]) -> String {
    let mut hasher = Sha256::new();
    for (key, value) in parts {
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }
    hex_bytes(&hasher.finalize())
}

fn optional_decimal(value: Option<Decimal>) -> String {
    value.map(|value| value.to_string()).unwrap_or_else(|| "none".into())
}

fn configured_cost(cost: Option<&ProviderModelCost>) -> String {
    let Some(cost) = cost else {
        return "none".into();
    };
    fingerprint(&[
        ("mode", serde_json::to_string(&cost.cost_mode).expect("cost mode must serialize")),
        ("price_per_request", optional_decimal(cost.price_per_request)),
        ("input_price_per_million", optional_decimal(cost.input_price_per_million)),
        ("output_price_per_million", optional_decimal(cost.output_price_per_million)),
        ("cache_creation_price_per_million", optional_decimal(cost.cache_creation_price_per_million)),
        ("cache_read_price_per_million", optional_decimal(cost.cache_read_price_per_million)),
    ])
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::{
        model::TieredPricingConfig,
        provider::{ProviderModelCost, ProviderModelCostMode},
    };

    use super::{PriceFingerprintInput, price_config_fingerprint};

    #[test]
    fn price_fingerprint_changes_when_multiplier_changes() {
        let tiered_pricing = TieredPricingConfig { tiers: Vec::new() };

        let low = price_config_fingerprint(PriceFingerprintInput {
            configured_cost: None,
            price_per_request: Some(Decimal::ONE),
            tiered_pricing: &tiered_pricing,
            billing_multiplier: Decimal::ONE,
        });
        let high = price_config_fingerprint(PriceFingerprintInput {
            configured_cost: None,
            price_per_request: Some(Decimal::ONE),
            tiered_pricing: &tiered_pricing,
            billing_multiplier: Decimal::from(2),
        });

        assert_ne!(low, high);
        assert_eq!(low.len(), 64);
    }

    #[test]
    fn price_fingerprint_changes_when_configured_cost_changes() {
        let tiered_pricing = TieredPricingConfig { tiers: Vec::new() };
        let cheap = configured_cost(Decimal::ONE);
        let expensive = configured_cost(Decimal::from(2));

        let cheap_fingerprint = price_config_fingerprint(PriceFingerprintInput {
            configured_cost: Some(&cheap),
            price_per_request: None,
            tiered_pricing: &tiered_pricing,
            billing_multiplier: Decimal::ONE,
        });
        let expensive_fingerprint = price_config_fingerprint(PriceFingerprintInput {
            configured_cost: Some(&expensive),
            price_per_request: None,
            tiered_pricing: &tiered_pricing,
            billing_multiplier: Decimal::ONE,
        });

        assert_ne!(cheap_fingerprint, expensive_fingerprint);
    }

    fn configured_cost(price_per_request: Decimal) -> ProviderModelCost {
        ProviderModelCost {
            id: "cost-a".into(),
            provider_id: "provider-a".into(),
            key_id: "key-a".into(),
            provider_model_id: "model-a".into(),
            cost_mode: ProviderModelCostMode::PerRequest,
            price_per_request: Some(price_per_request),
            input_price_per_million: None,
            output_price_per_million: None,
            cache_creation_price_per_million: None,
            cache_read_price_per_million: None,
            created_at: "2026-06-16T00:00:00Z".into(),
            updated_at: "2026-06-16T00:00:00Z".into(),
        }
    }
}
