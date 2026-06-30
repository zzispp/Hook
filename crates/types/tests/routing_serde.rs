use types::provider::{
    RouteScoreExplanation, RoutingMetricSource, RoutingPriorSource, RoutingProfile, RoutingProfileId, RoutingRequestSizeBucket,
    default_contextual_exploration_enabled, default_ema_alpha, default_ema_max_freshness_seconds, default_ema_recent_cap, default_ema_recent_weight,
    default_exploration_cap, default_exploration_min_success_score, default_exploration_weight, default_prior_sample_cap,
};

#[test]
fn decision_candidates_without_metric_window_deserialize_with_default_window() {
    let payload = r#"{
      "route": {
        "provider_id": "provider",
        "key_id": "key",
        "endpoint_id": "endpoint",
        "global_model_id": "model",
        "client_api_format": "openai:chat",
        "provider_api_format": "openai:chat",
        "is_stream": false
      },
      "provider_name": "provider",
      "key_name": "key",
      "key_preview": "key",
      "endpoint_name": "openai:chat",
      "rank": 1,
      "state": "warming",
      "final_score": 75.895,
      "selected_reason": "score 75.9",
      "components": [],
      "raw_metrics": {
        "request_count": 0,
        "success_count": 0,
        "failure_count": 0,
        "timeout_count": 0,
        "rate_limited_count": 0,
        "server_error_count": 0,
        "latency_avg_ms": null,
        "first_token_avg_ms": null,
        "output_tps": null,
        "upstream_total_cost": null,
        "total_tokens": 0,
        "sample_count": 0,
        "rpm_used": 0,
        "rpm_limit": null
      },
      "exclusion_reason": null,
      "metric_freshness_seconds": 0
    }"#;

    let explanation: RouteScoreExplanation = serde_json::from_str(payload).expect("legacy routing decision payload should deserialize");
    assert_eq!(explanation.metric_window.as_str(), "5m");
    assert_eq!(explanation.metric_source, RoutingMetricSource::Unknown);
    assert_eq!(explanation.prior_source, RoutingPriorSource::Unknown);
    assert_eq!(explanation.prior_sample_count, 0);
    assert_eq!(explanation.request_features.request_size_bucket, RoutingRequestSizeBucket::Unknown);
}

#[test]
fn routing_profile_payload_deserializes_new_tuning_defaults() {
    let payload = r#"{
      "id": "balanced",
      "name": "Balanced",
        "description": "current payload",
      "weights": {
        "success": 0.28,
        "first_token": 0.19,
        "latency": 0.17,
        "tps": 0.09,
        "cost": 0.15,
        "headroom": 0.12,
        "priority": 0.0
      },
      "version": "legacy-v1",
      "min_samples": 20,
      "exploration_k": 3.0,
      "conversion_penalty": 6.0,
      "stale_metric_penalty": 8.0,
      "affinity_bonus": 6.0,
      "auto_tune_enabled": true
    }"#;

    let profile: RoutingProfile = serde_json::from_str(payload).expect("routing profile payload should deserialize");

    assert_eq!(profile.id, RoutingProfileId::Balanced);
    assert_eq!(profile.prior_sample_cap, default_prior_sample_cap());
    assert_eq!(profile.contextual_exploration_enabled, default_contextual_exploration_enabled());
    assert_eq!(profile.ema_alpha, default_ema_alpha());
    assert_eq!(profile.ema_max_freshness_seconds, default_ema_max_freshness_seconds());
    assert_eq!(profile.ema_recent_weight, default_ema_recent_weight());
    assert_eq!(profile.ema_recent_cap, default_ema_recent_cap());
    assert_eq!(profile.exploration_weight, default_exploration_weight());
    assert_eq!(profile.exploration_cap, default_exploration_cap());
    assert_eq!(profile.exploration_min_success_score, default_exploration_min_success_score());
}
