use types::provider::RouteScoreExplanation;

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
        "ttfb_avg_ms": null,
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
}
