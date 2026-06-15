use std::collections::BTreeMap;

use provider::application::billing::{normalized_default_dimensions, quantize};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use storage::provider::ProviderStore;
use types::model::{PricingTier, TieredPricingConfig};
use types::provider::{ProviderModelCost, ProviderModelCostMode, ProviderModelCostSource, RequestUpstreamCost};

use super::{AttemptAuditInput, TokenUsage, billing_runtime::total_tokens};
use crate::llm_proxy::LlmProxyError;

const PRICE_SCALE: i64 = 1_000_000;

#[derive(Clone, Debug)]
struct UpstreamCostPrices {
    source: ProviderModelCostSource,
    mode: ProviderModelCostMode,
    price_per_request: Option<Decimal>,
    input_price_per_million: Option<Decimal>,
    output_price_per_million: Option<Decimal>,
    cache_creation_price_per_million: Option<Decimal>,
    cache_read_price_per_million: Option<Decimal>,
}

pub(super) async fn request_upstream_cost(store: &ProviderStore, input: &AttemptAuditInput) -> Result<RequestUpstreamCost, LlmProxyError> {
    if input.status != "success" {
        return Ok(RequestUpstreamCost::default());
    }
    let configured = store
        .find_model_cost(&input.candidate.trace.key_id, &input.candidate.trace.provider_model_id)
        .await?;
    let prices = match configured {
        Some(cost) => configured_prices(cost),
        None => global_default_prices(input),
    };
    Ok(match prices.mode {
        ProviderModelCostMode::PerRequest => per_request_cost(prices),
        ProviderModelCostMode::PerToken => per_token_cost(prices, input),
    })
}

fn configured_prices(cost: ProviderModelCost) -> UpstreamCostPrices {
    UpstreamCostPrices {
        source: ProviderModelCostSource::Configured,
        mode: cost.cost_mode,
        price_per_request: cost.price_per_request,
        input_price_per_million: cost.input_price_per_million,
        output_price_per_million: cost.output_price_per_million,
        cache_creation_price_per_million: cost.cache_creation_price_per_million,
        cache_read_price_per_million: cost.cache_read_price_per_million,
    }
}

fn global_default_prices(input: &AttemptAuditInput) -> UpstreamCostPrices {
    if let Some(tier) = selected_global_tier(input) {
        return token_prices(ProviderModelCostSource::GlobalDefault, tier);
    }
    UpstreamCostPrices {
        source: ProviderModelCostSource::GlobalDefault,
        mode: ProviderModelCostMode::PerRequest,
        price_per_request: input.candidate.price_per_request,
        input_price_per_million: None,
        output_price_per_million: None,
        cache_creation_price_per_million: None,
        cache_read_price_per_million: None,
    }
}

fn token_prices(source: ProviderModelCostSource, tier: &PricingTier) -> UpstreamCostPrices {
    UpstreamCostPrices {
        source,
        mode: ProviderModelCostMode::PerToken,
        price_per_request: None,
        input_price_per_million: Some(tier.input_price_per_1m),
        output_price_per_million: Some(tier.output_price_per_1m),
        cache_creation_price_per_million: Some(cache_creation_price(tier)),
        cache_read_price_per_million: Some(cache_read_price(tier)),
    }
}

fn per_request_cost(prices: UpstreamCostPrices) -> RequestUpstreamCost {
    let request_cost = quantize(prices.price_per_request.unwrap_or(Decimal::ZERO));
    RequestUpstreamCost {
        upstream_cost_mode: Some(ProviderModelCostMode::PerRequest),
        upstream_cost_source: Some(prices.source),
        upstream_price_per_request: prices.price_per_request,
        upstream_request_cost: Some(request_cost),
        upstream_total_cost: Some(request_cost),
        ..RequestUpstreamCost::default()
    }
}

fn per_token_cost(prices: UpstreamCostPrices, input: &AttemptAuditInput) -> RequestUpstreamCost {
    let usage = normalized_usage(input);
    let input_cost = token_cost(usage.input_tokens, prices.input_price_per_million);
    let output_cost = token_cost(usage.output_tokens, prices.output_price_per_million);
    let cache_creation_cost = token_cost(usage.cache_creation_tokens, prices.cache_creation_price_per_million);
    let cache_read_cost = token_cost(usage.cache_read_tokens, prices.cache_read_price_per_million);
    RequestUpstreamCost {
        upstream_cost_mode: Some(ProviderModelCostMode::PerToken),
        upstream_cost_source: Some(prices.source),
        upstream_input_price_per_million: prices.input_price_per_million,
        upstream_output_price_per_million: prices.output_price_per_million,
        upstream_cache_creation_price_per_million: prices.cache_creation_price_per_million,
        upstream_cache_read_price_per_million: prices.cache_read_price_per_million,
        upstream_input_cost: Some(input_cost),
        upstream_output_cost: Some(output_cost),
        upstream_cache_creation_cost: Some(cache_creation_cost),
        upstream_cache_read_cost: Some(cache_read_cost),
        upstream_total_cost: Some(quantize(input_cost + output_cost + cache_creation_cost + cache_read_cost)),
        ..RequestUpstreamCost::default()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct NormalizedUsage {
    input_tokens: i64,
    output_tokens: i64,
    cache_creation_tokens: i64,
    cache_read_tokens: i64,
}

fn normalized_usage(input: &AttemptAuditInput) -> NormalizedUsage {
    let dimensions = normalized_default_dimensions(&input.candidate.trace.provider_api_format, usage_dimensions(input.usage));
    NormalizedUsage {
        input_tokens: int_dim(&dimensions, "input_tokens"),
        output_tokens: int_dim(&dimensions, "output_tokens"),
        cache_creation_tokens: int_dim(&dimensions, "cache_creation_tokens"),
        cache_read_tokens: int_dim(&dimensions, "cache_read_tokens"),
    }
}

fn usage_dimensions(usage: Option<TokenUsage>) -> BTreeMap<String, Value> {
    let mut dimensions = BTreeMap::new();
    if let Some(usage) = usage {
        insert_i64(&mut dimensions, "input_tokens", usage.prompt_tokens);
        insert_i64(&mut dimensions, "output_tokens", usage.completion_tokens);
        insert_i64(&mut dimensions, "total_tokens", total_tokens(Some(usage)));
        insert_i64(&mut dimensions, "cache_creation_input_tokens", usage.cache_creation_input_tokens);
        insert_i64(&mut dimensions, "cache_read_input_tokens", usage.cache_read_input_tokens);
    }
    dimensions
}

fn selected_global_tier(input: &AttemptAuditInput) -> Option<&PricingTier> {
    let pricing = &input.candidate.tiered_pricing;
    let total_context = normalized_default_dimensions(&input.candidate.trace.provider_api_format, usage_dimensions(input.usage))
        .get("total_input_context")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    matching_tier(pricing, total_context)
}

fn matching_tier(pricing: &TieredPricingConfig, total_context: i64) -> Option<&PricingTier> {
    pricing
        .tiers
        .iter()
        .find(|tier| tier.up_to.is_none_or(|limit| total_context <= limit.try_into().unwrap_or(i64::MAX)))
        .or_else(|| pricing.tiers.last())
}

fn cache_creation_price(tier: &PricingTier) -> Decimal {
    tier.cache_creation_price_per_1m.unwrap_or(tier.input_price_per_1m * Decimal::new(125, 2))
}

fn cache_read_price(tier: &PricingTier) -> Decimal {
    tier.cache_read_price_per_1m
        .or_else(|| {
            tier.cache_ttl_pricing
                .as_ref()?
                .iter()
                .find(|item| item.ttl_minutes == 5)
                .and_then(|item| item.cache_read_price_per_1m)
        })
        .unwrap_or(tier.input_price_per_1m * Decimal::new(1, 1))
}

fn token_cost(tokens: i64, price: Option<Decimal>) -> Decimal {
    quantize(Decimal::from(tokens.max(0)) * price.unwrap_or(Decimal::ZERO) / Decimal::from(PRICE_SCALE))
}

fn insert_i64(dimensions: &mut BTreeMap<String, Value>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        dimensions.insert(key.into(), json!(value));
    }
}

fn int_dim(dimensions: &BTreeMap<String, Value>, key: &str) -> i64 {
    dimensions.get(key).and_then(Value::as_i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use provider::application::billing::quantize;
    use rust_decimal::Decimal;
    use types::model::{PricingTier, TieredPricingConfig};
    use types::provider::{ProviderModelCostMode, ProviderModelCostSource};

    use super::{matching_tier, per_token_cost, token_prices};
    use crate::llm_proxy::{
        audit::{AttemptAuditInput, AttemptRecordInput, TokenUsage},
        candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    };

    #[test]
    fn per_token_cost_removes_openai_cache_read_tokens_from_input_cost() {
        let candidate = candidate_with_pricing(TieredPricingConfig {
            tiers: vec![PricingTier {
                up_to: None,
                input_price_per_1m: Decimal::new(2, 0),
                output_price_per_1m: Decimal::new(4, 0),
                cache_creation_price_per_1m: None,
                cache_read_price_per_1m: Some(Decimal::new(1, 0)),
                cache_ttl_pricing: None,
            }],
        });
        let input = attempt_input(&candidate, 1_000, 100, Some(300), Some(200));
        let upstream = per_token_cost(token_prices(ProviderModelCostSource::GlobalDefault, &candidate.tiered_pricing.tiers[0]), &input);

        assert_eq!(upstream.upstream_cost_mode, Some(ProviderModelCostMode::PerToken));
        assert_eq!(upstream.upstream_cost_source, Some(ProviderModelCostSource::GlobalDefault));
        assert_eq!(upstream.upstream_input_cost, Some(Decimal::new(160000, 8)));
        assert_eq!(upstream.upstream_cache_creation_cost, Some(Decimal::new(75000, 8)));
        assert_eq!(upstream.upstream_cache_read_cost, Some(Decimal::new(20000, 8)));
        assert_eq!(upstream.upstream_total_cost, Some(quantize(Decimal::new(295000, 8))));
    }

    #[test]
    fn matching_tier_uses_total_input_context() {
        let pricing = TieredPricingConfig {
            tiers: vec![
                PricingTier {
                    up_to: Some(100),
                    input_price_per_1m: Decimal::ONE,
                    output_price_per_1m: Decimal::ONE,
                    cache_creation_price_per_1m: None,
                    cache_read_price_per_1m: None,
                    cache_ttl_pricing: None,
                },
                PricingTier {
                    up_to: None,
                    input_price_per_1m: Decimal::new(2, 0),
                    output_price_per_1m: Decimal::new(2, 0),
                    cache_creation_price_per_1m: None,
                    cache_read_price_per_1m: None,
                    cache_ttl_pricing: None,
                },
            ],
        };

        let tier = matching_tier(&pricing, 101).expect("tier");

        assert_eq!(tier.input_price_per_1m, Decimal::new(2, 0));
    }

    fn attempt_input(
        candidate: &ProxyCandidate,
        input_tokens: i64,
        output_tokens: i64,
        cache_creation_tokens: Option<i64>,
        cache_read_tokens: Option<i64>,
    ) -> AttemptAuditInput {
        AttemptAuditInput::from(AttemptRecordInput {
            usage: Some(TokenUsage {
                prompt_tokens: Some(input_tokens),
                completion_tokens: Some(output_tokens),
                cache_creation_input_tokens: cache_creation_tokens,
                cache_read_input_tokens: cache_read_tokens,
                ..TokenUsage::default()
            }),
            ..AttemptRecordInput::new(candidate, 0, "success", true)
        })
    }

    fn candidate_with_pricing(tiered_pricing: TieredPricingConfig) -> ProxyCandidate {
        ProxyCandidate {
            trace: trace(),
            requested_model_name: "gpt-test".into(),
            api_key: "secret".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            upstream_url: "https://example.com/v1/chat/completions".into(),
            provider_model_name: "gpt-test".into(),
            reasoning_effort: None,
            header_rules: None,
            body_rules: None,
            price_per_request: None,
            tiered_pricing,
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: Some(300.0),
            stream_first_byte_timeout_seconds: Some(30.0),
            stream_idle_timeout_seconds: Some(30.0),
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            cache_affinity_enabled: false,
            is_cached: false,
            route: CandidateRoute { options: Vec::new() },
        }
    }

    fn trace() -> CandidateTrace {
        CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("alice".into()),
            token_name_snapshot: Some("token".into()),
            token_prefix_snapshot: Some("sk-test".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-test".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "Provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "endpoint".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "Key".into(),
            key_preview_snapshot: "***test".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: "openai:chat".into(),
            needs_conversion: false,
            is_stream: false,
            is_cached: false,
            candidate_index: 0,
        }
    }
}
