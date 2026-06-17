use std::collections::HashSet;

use crate::llm_proxy::{
    LlmProxyError,
    cache::snapshot::{CachedEndpoint, CachedProviderKey},
    routing::ScoredRoute,
};

use super::super::CandidateParts;

#[derive(Clone, Copy)]
pub(super) struct ScoreTarget {
    pub(super) part_index: usize,
    pub(super) endpoint_index: usize,
}

pub(super) fn score_targets(parts: &[CandidateParts]) -> Vec<ScoreTarget> {
    parts
        .iter()
        .enumerate()
        .flat_map(|(part_index, part)| {
            part.endpoints
                .iter()
                .enumerate()
                .filter(|(_, endpoint)| key_for_endpoint(part, endpoint).is_ok())
                .map(move |(endpoint_index, _)| ScoreTarget { part_index, endpoint_index })
        })
        .collect()
}

pub(super) fn ordered_parts(parts: Vec<CandidateParts>, targets: &[ScoreTarget], scores: &[ScoredRoute]) -> Result<Vec<CandidateParts>, LlmProxyError> {
    let mut plans = vec![PartPlan::default(); parts.len()];
    for (score_order, score) in scores.iter().enumerate().filter(|(_, item)| !item.excluded) {
        let target = targets
            .get(score.original_index)
            .ok_or_else(|| LlmProxyError::Infrastructure("dynamic routing score referenced an invalid route option index".into()))?;
        plans[target.part_index].push(target.endpoint_index, score_order);
    }
    let mut ranked_indices = ranked_part_indices(&plans);
    ranked_indices.sort_by_key(|index| plans[*index].first_score_order);
    ranked_indices.into_iter().map(|index| ordered_part(parts.get(index), &plans[index])).collect()
}

pub(super) fn part(parts: &[CandidateParts], target: ScoreTarget) -> Result<&CandidateParts, LlmProxyError> {
    parts
        .get(target.part_index)
        .ok_or_else(|| LlmProxyError::Infrastructure("dynamic routing route option referenced an invalid candidate index".into()))
}

pub(super) fn endpoint(part: &CandidateParts, target: ScoreTarget) -> Result<&CachedEndpoint, LlmProxyError> {
    part.endpoints
        .get(target.endpoint_index)
        .ok_or_else(|| LlmProxyError::Infrastructure("dynamic routing route option referenced an invalid endpoint index".into()))
}

pub(super) fn key_for_endpoint<'a>(part: &'a CandidateParts, endpoint: &CachedEndpoint) -> Result<&'a CachedProviderKey, LlmProxyError> {
    part.keys
        .iter()
        .find(|key| key.api_formats.iter().any(|format| format == &endpoint.api_format))
        .ok_or_else(|| LlmProxyError::Infrastructure("candidate parts must contain at least one key for a scored endpoint".into()))
}

fn ranked_part_indices(plans: &[PartPlan]) -> Vec<usize> {
    plans
        .iter()
        .enumerate()
        .filter(|(_, plan)| plan.first_score_order.is_some())
        .map(|(index, _)| index)
        .collect()
}

fn ordered_part(part: Option<&CandidateParts>, plan: &PartPlan) -> Result<CandidateParts, LlmProxyError> {
    let part = part.ok_or_else(|| LlmProxyError::Infrastructure("dynamic routing score referenced an invalid candidate index".into()))?;
    let mut output = part.clone();
    output.endpoints = plan
        .endpoint_indices
        .iter()
        .map(|index| {
            endpoint(
                part,
                ScoreTarget {
                    part_index: 0,
                    endpoint_index: *index,
                },
            )
            .cloned()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(output)
}

#[derive(Clone, Default)]
struct PartPlan {
    endpoint_indices: Vec<usize>,
    seen_endpoint_indices: HashSet<usize>,
    first_score_order: Option<usize>,
}

impl PartPlan {
    fn push(&mut self, endpoint_index: usize, score_order: usize) {
        self.first_score_order = self.first_score_order.or(Some(score_order));
        if self.seen_endpoint_indices.insert(endpoint_index) {
            self.endpoint_indices.push(endpoint_index);
        }
    }
}

#[cfg(test)]
mod tests {
    use types::provider::{RouteIdentity, RouteScoreExplanation, RoutingMetricSnapshot, RoutingMetricWindow, RoutingRouteState};

    use super::*;
    use crate::llm_proxy::cache::snapshot::{CachedModelBinding, CachedProvider};

    #[test]
    fn ordered_parts_keeps_only_scored_eligible_endpoints_in_score_order() {
        let parts = vec![candidate_part()];
        let targets = score_targets(&parts);
        let scores = vec![score(1, false), score(0, false), score(2, true)];

        let ordered = ordered_parts(parts, &targets, &scores).unwrap();

        let endpoint_ids = ordered[0].endpoints.iter().map(|endpoint| endpoint.id.as_str()).collect::<Vec<_>>();
        assert_eq!(endpoint_ids, vec!["endpoint-fast", "endpoint-slow"]);
    }

    #[test]
    fn ordered_parts_keeps_secondary_when_primary_is_excluded() {
        let parts = vec![candidate_part()];
        let targets = score_targets(&parts);
        let scores = vec![score(0, true), score(1, false)];

        let ordered = ordered_parts(parts, &targets, &scores).unwrap();

        let endpoint_ids = ordered[0].endpoints.iter().map(|endpoint| endpoint.id.as_str()).collect::<Vec<_>>();
        assert_eq!(endpoint_ids, vec!["endpoint-fast"]);
    }

    #[test]
    fn ordered_parts_drops_excluded_secondary_retry_option() {
        let parts = vec![candidate_part()];
        let targets = score_targets(&parts);
        let scores = vec![score(0, false), score(1, true)];

        let ordered = ordered_parts(parts, &targets, &scores).unwrap();

        let endpoint_ids = ordered[0].endpoints.iter().map(|endpoint| endpoint.id.as_str()).collect::<Vec<_>>();
        assert_eq!(endpoint_ids, vec!["endpoint-slow"]);
    }

    fn candidate_part() -> CandidateParts {
        CandidateParts {
            provider: CachedProvider {
                id: "provider-a".into(),
                name: "Provider A".into(),
                max_retries: None,
                request_timeout_seconds: None,
                stream_first_byte_timeout_seconds: None,
                stream_idle_timeout_seconds: None,
                priority: 10,
                keep_priority_on_conversion: false,
                enable_format_conversion: true,
                is_active: true,
                endpoints: Vec::new(),
                keys: Vec::new(),
                models: Vec::new(),
            },
            endpoints: vec![endpoint("endpoint-slow"), endpoint("endpoint-fast"), endpoint("endpoint-excluded")],
            keys: vec![key()],
            model: CachedModelBinding {
                id: "binding-a".into(),
                provider_id: "provider-a".into(),
                global_model_id: "model-a".into(),
                is_active: true,
            },
            effective_upstream_model_name: "upstream-model".into(),
            effective_reasoning_effort: None,
            client_api_format: "openai:chat".into(),
            routing_api_format: "openai:chat".into(),
            is_cached: false,
        }
    }

    fn endpoint(id: &str) -> CachedEndpoint {
        CachedEndpoint {
            id: id.into(),
            provider_id: "provider-a".into(),
            api_format: "openai:chat".into(),
            base_url: "https://example.com".into(),
            custom_path: None,
            max_retries: None,
            is_active: true,
            format_acceptance_config: None,
            header_rules: None,
            body_rules: None,
        }
    }

    fn key() -> CachedProviderKey {
        CachedProviderKey {
            id: "key-a".into(),
            provider_id: "provider-a".into(),
            name: "Key A".into(),
            api_formats: vec!["openai:chat".into()],
            allowed_model_ids: Vec::new(),
            capabilities: None,
            key_preview: "key-a".into(),
            encrypted_api_key: "encrypted".into(),
            internal_priority: 10,
            global_priority_by_format: Default::default(),
            rpm_limit: None,
            cache_ttl_minutes: 5,
            time_range_enabled: false,
            time_range_start_minute: None,
            time_range_end_minute: None,
            is_active: true,
            model_mappings: Default::default(),
        }
    }

    fn score(original_index: usize, excluded: bool) -> ScoredRoute {
        ScoredRoute {
            original_index,
            excluded,
            explanation: RouteScoreExplanation {
                route: RouteIdentity {
                    provider_id: "provider-a".into(),
                    key_id: "key-a".into(),
                    endpoint_id: format!("endpoint-{original_index}"),
                    global_model_id: "model-a".into(),
                    client_api_format: "openai:chat".into(),
                    provider_api_format: "openai:chat".into(),
                    is_stream: false,
                },
                provider_name: None,
                key_name: None,
                key_preview: None,
                endpoint_name: None,
                rank: 0,
                state: RoutingRouteState::Eligible,
                final_score: 0.0,
                metric_window: RoutingMetricWindow::FiveMinutes,
                selected_reason: String::new(),
                components: Vec::new(),
                raw_metrics: RoutingMetricSnapshot::default(),
                exclusion_reason: None,
                metric_freshness_seconds: 0,
                metric_source: Default::default(),
                prior_source: Default::default(),
                prior_sample_count: 0,
                effective_sample_count: 0,
                routing_context_key: None,
                route_config_fingerprint: None,
                price_config_fingerprint: None,
                request_features: Default::default(),
            },
        }
    }
}
