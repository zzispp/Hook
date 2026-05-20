use provider::application::billing::{BillingSnapshot, BillingSnapshotStatus, RequestBillingAmount};
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::BTreeMap;
use types::model::TieredPricingConfig;

use super::{AttemptRecordInput, billing_runtime::BillingAttempt, request_billing_status, token_usage_record, wallet_settlement_input};
use crate::llm_proxy::candidate::{CandidateRoute, CandidateTrace, ProxyCandidate};

#[test]
fn success_without_usage_still_settles_request_only_billing() {
    let candidate = candidate();
    let input = AttemptRecordInput::new(&candidate, 0, "success", true);
    let billing = request_only_billing();

    let usage_record = token_usage_record("request-1", &input, Some(&billing), time::OffsetDateTime::UNIX_EPOCH).unwrap();
    let settlement = wallet_settlement_input("request-1", &input, Some(&billing)).unwrap();

    assert_eq!(request_billing_status(&input, Some(&billing)), "settled");
    assert_eq!(usage_record.expect("usage record").cost, Decimal::new(5, 1));
    assert_eq!(settlement.expect("settlement").amount.total_cost, Decimal::new(5, 1));
}

#[test]
fn incomplete_billing_sets_status_and_blocks_settlement() {
    let candidate = candidate();
    let input = AttemptRecordInput {
        usage: Some(super::TokenUsage {
            prompt_tokens: Some(10),
            completion_tokens: Some(5),
            ..Default::default()
        }),
        ..AttemptRecordInput::new(&candidate, 0, "success", true)
    };
    let billing = incomplete_billing();

    let settlement = wallet_settlement_input("request-1", &input, Some(&billing));

    assert_eq!(request_billing_status(&input, Some(&billing)), "billing_incomplete");
    assert!(settlement.is_err());
}

#[test]
fn estimated_stream_usage_can_settle_billing() {
    let candidate = candidate();
    let input = AttemptRecordInput {
        usage: Some(super::TokenUsage {
            prompt_tokens: Some(10),
            completion_tokens: Some(5),
            total_tokens: Some(15),
            usage_source: Some("estimated_from_stream_delta"),
            usage_semantic: Some("responses"),
            ..Default::default()
        }),
        ..AttemptRecordInput::new(&candidate, 0, "success", true)
    };
    let billing = complete_billing();

    let usage_record = token_usage_record("request-1", &input, Some(&billing), time::OffsetDateTime::UNIX_EPOCH).unwrap();
    let settlement = wallet_settlement_input("request-1", &input, Some(&billing)).unwrap();

    assert_eq!(request_billing_status(&input, Some(&billing)), "settled");
    assert_eq!(usage_record.expect("usage record").cost, Decimal::ONE);
    assert_eq!(settlement.expect("settlement").amount.total_cost, Decimal::ONE);
}

fn candidate() -> ProxyCandidate {
    ProxyCandidate {
        trace: trace(),
        requested_model_name: "gpt-5.5".into(),
        api_key: "secret".into(),
        base_url: "https://example.com".into(),
        custom_path: None,
        upstream_url: "https://example.com/v1/responses".into(),
        provider_model_name: "gpt-5.5".into(),
        reasoning_effort: None,
        header_rules: None,
        body_rules: None,
        price_per_request: None,
        tiered_pricing: TieredPricingConfig { tiers: Vec::new() },
        billing_multiplier: Decimal::ONE,
        max_retries: 0,
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(30.0),
        stream_idle_timeout_seconds: Some(30.0),
        cache_ttl_minutes: 5,
        key_rpm_limit: None,
        is_cached: false,
        route: CandidateRoute { options: Vec::new() },
    }
}

fn complete_billing() -> BillingAttempt {
    BillingAttempt {
        amount: RequestBillingAmount {
            input_cost: Decimal::ZERO,
            output_cost: Decimal::ONE,
            cache_creation_cost: Decimal::ZERO,
            cache_read_cost: Decimal::ZERO,
            request_cost: Decimal::ZERO,
            token_cost: Decimal::ONE,
            base_cost: Decimal::ONE,
            total_cost: Decimal::ONE,
            billing_multiplier: Decimal::ONE,
            input_price_per_1m: None,
            output_price_per_1m: None,
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            currency: "USD".into(),
            snapshot: complete_snapshot(),
        },
        snapshot: json!({"status": "complete"}),
        status: BillingSnapshotStatus::Complete,
    }
}

fn request_only_billing() -> BillingAttempt {
    BillingAttempt {
        amount: RequestBillingAmount {
            input_cost: Decimal::ZERO,
            output_cost: Decimal::ZERO,
            cache_creation_cost: Decimal::ZERO,
            cache_read_cost: Decimal::ZERO,
            request_cost: Decimal::new(5, 1),
            token_cost: Decimal::ZERO,
            base_cost: Decimal::new(5, 1),
            total_cost: Decimal::new(5, 1),
            billing_multiplier: Decimal::ONE,
            input_price_per_1m: None,
            output_price_per_1m: None,
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            currency: "USD".into(),
            snapshot: request_only_snapshot(),
        },
        snapshot: json!({"status": "complete"}),
        status: BillingSnapshotStatus::Complete,
    }
}

fn incomplete_billing() -> BillingAttempt {
    BillingAttempt {
        amount: RequestBillingAmount {
            input_cost: Decimal::ZERO,
            output_cost: Decimal::ZERO,
            cache_creation_cost: Decimal::ZERO,
            cache_read_cost: Decimal::ZERO,
            request_cost: Decimal::ZERO,
            token_cost: Decimal::ZERO,
            base_cost: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            billing_multiplier: Decimal::ONE,
            input_price_per_1m: None,
            output_price_per_1m: None,
            cache_creation_price_per_1m: None,
            cache_read_price_per_1m: None,
            currency: "USD".into(),
            snapshot: incomplete_snapshot(),
        },
        snapshot: json!({"status": "incomplete"}),
        status: BillingSnapshotStatus::Incomplete,
    }
}

fn complete_snapshot() -> BillingSnapshot {
    BillingSnapshot {
        missing_required: Vec::new(),
        status: BillingSnapshotStatus::Complete,
        ..incomplete_snapshot()
    }
}

fn incomplete_snapshot() -> BillingSnapshot {
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
        missing_required: vec!["input_tokens".into()],
        status: BillingSnapshotStatus::Incomplete,
        calculated_at: "2026-05-17T00:00:00Z".into(),
        engine_version: "2.0".into(),
    }
}

fn request_only_snapshot() -> BillingSnapshot {
    BillingSnapshot {
        cost_breakdown: BTreeMap::from([("request_cost".into(), Decimal::new(5, 1))]),
        resolved_dimensions: BTreeMap::from([("request_count".into(), json!(1))]),
        resolved_variables: BTreeMap::from([("price_per_request".into(), json!(Decimal::new(5, 1)))]),
        base_total_cost: Decimal::new(5, 1),
        total_cost: Decimal::new(5, 1),
        missing_required: Vec::new(),
        status: BillingSnapshotStatus::Complete,
        ..incomplete_snapshot()
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
        model_name_snapshot: "gpt-5.5".into(),
        provider_id: "provider-1".into(),
        provider_name_snapshot: "Provider".into(),
        endpoint_id: "endpoint-1".into(),
        endpoint_name_snapshot: "endpoint".into(),
        key_id: "key-1".into(),
        key_name_snapshot: "Key".into(),
        key_preview_snapshot: "***test".into(),
        client_api_format: "openai_cli".into(),
        provider_api_format: "openai_cli".into(),
        needs_conversion: false,
        is_stream: true,
        is_cached: false,
        candidate_index: 0,
    }
}
