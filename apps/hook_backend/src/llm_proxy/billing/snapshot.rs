use types::api_token::ApiToken;

use super::WalletSettlementInput;

pub(super) struct DescriptionInput<'a, 'b> {
    pub(super) input: &'a WalletSettlementInput<'b>,
    pub(super) token: &'a ApiToken,
}

pub(super) fn settlement_description(input: DescriptionInput<'_, '_>) -> String {
    format!(
        "Model: {}; Cost: {} {}; API endpoint: {}; Token: {} ({})",
        input.input.candidate.trace.model_name_snapshot,
        input.input.amount.total_cost,
        input.input.amount.currency,
        input.input.candidate.trace.client_api_format,
        input.token.name,
        input.token.token_prefix
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use provider::application::billing::{BillingSnapshot, BillingSnapshotStatus, RequestBillingAmount};
    use rust_decimal::Decimal;
    use types::{
        api_token::{ApiToken, ApiTokenType, ModelAccessMode},
        model::TieredPricingConfig,
    };

    use super::*;
    use crate::llm_proxy::{
        audit::AuditCandidate,
        candidate::{CandidateRoute, CandidateTrace, ProxyCandidate},
    };

    #[test]
    fn settlement_description_describes_public_usage_without_internal_snapshot() {
        let candidate = candidate();
        let audit_candidate = AuditCandidate::from(&candidate);
        let input = WalletSettlementInput {
            request_id: "request-1",
            candidate: &audit_candidate,
            amount: billing_amount(),
        };

        let description = settlement_description(DescriptionInput {
            input: &input,
            token: &token(),
        });

        assert_eq!(
            description,
            "Model: gpt-5.5; Cost: 0.00270 USD; API endpoint: openai:chat; Token: free (sk-test)"
        );
        assert!(!description.contains('{'));
        assert!(!description.contains("provider"));
        assert!(!description.contains("upstream-key"));
    }

    fn candidate() -> ProxyCandidate {
        ProxyCandidate {
            trace: trace(),
            requested_model_name: "gpt-5.5".into(),
            api_key: "secret".into(),
            base_url: "https://internal.example.com".into(),
            custom_path: None,
            upstream_url: "https://internal.example.com/v1/chat/completions".into(),
            provider_model_name: "upstream-model".into(),
            reasoning_effort: None,
            header_rules: None,
            body_rules: None,
            key_supports_image_generation: false,
            price_per_request: None,
            tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
            billing_multiplier: Decimal::ONE,
            max_retries: 0,
            request_timeout_seconds: None,
            stream_first_byte_timeout_seconds: None,
            stream_idle_timeout_seconds: None,
            cache_ttl_minutes: 5,
            key_rpm_limit: None,
            is_cached: false,
            route: CandidateRoute { options: Vec::new() },
        }
    }

    fn trace() -> CandidateTrace {
        CandidateTrace {
            token_id: Some("token-1".into()),
            user_id_snapshot: Some("user-1".into()),
            username_snapshot: Some("demo".into()),
            token_name_snapshot: Some("free".into()),
            token_prefix_snapshot: Some("sk-test".into()),
            group_code: Some("default".into()),
            global_model_id: "model-1".into(),
            provider_model_id: "provider-model-1".into(),
            model_name_snapshot: "gpt-5.5".into(),
            provider_id: "provider-1".into(),
            provider_name_snapshot: "internal-provider".into(),
            endpoint_id: "endpoint-1".into(),
            endpoint_name_snapshot: "internal-endpoint".into(),
            key_id: "key-1".into(),
            key_name_snapshot: "upstream-key".into(),
            key_preview_snapshot: "sk-upstream".into(),
            client_api_format: "openai:chat".into(),
            provider_api_format: "provider_chat".into(),
            needs_conversion: false,
            is_stream: false,
            is_cached: false,
            routing_context_key: "group=default|model=model-1|format=openai:chat|stream=false|size=unknown|cap=none".into(),
            route_config_fingerprint: "route-fingerprint".into(),
            price_config_fingerprint: "price-fingerprint".into(),
            candidate_index: 0,
        }
    }

    fn billing_amount() -> RequestBillingAmount {
        RequestBillingAmount {
            input_cost: Decimal::ZERO,
            output_cost: Decimal::ZERO,
            cache_creation_cost: Decimal::ZERO,
            cache_read_cost: Decimal::ZERO,
            request_cost: Decimal::ZERO,
            token_cost: Decimal::new(270, 5),
            base_cost: Decimal::new(270, 5),
            total_cost: Decimal::new(270, 5),
            billing_multiplier: Decimal::ONE,
            input_price_per_1m: None,
            output_price_per_1m: None,
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            currency: "USD".into(),
            snapshot: billing_snapshot(),
        }
    }

    fn billing_snapshot() -> BillingSnapshot {
        BillingSnapshot {
            schema_version: "2.0".into(),
            rule_id: Some("rule-1".into()),
            rule_name: Some("rule".into()),
            scope: Some("model".into()),
            expression: Some("input_cost".into()),
            resolved_dimensions: BTreeMap::new(),
            resolved_variables: BTreeMap::new(),
            cost_breakdown: BTreeMap::new(),
            base_total_cost: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            group_code: Some("default".into()),
            billing_multiplier: Decimal::ONE,
            tier_index: None,
            tier_info: None,
            missing_required: Vec::new(),
            status: BillingSnapshotStatus::Complete,
            calculated_at: "2026-05-19T00:00:00Z".into(),
            engine_version: "2.0".into(),
        }
    }

    fn token() -> ApiToken {
        ApiToken {
            id: "token-1".into(),
            user_id: Some("user-1".into()),
            token_type: ApiTokenType::User,
            name: "free".into(),
            token_value: String::new(),
            token_hash: String::new(),
            token_prefix: "sk-test".into(),
            group_code: "default".into(),
            expires_at: None,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm: None,
            quota_limit: None,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
