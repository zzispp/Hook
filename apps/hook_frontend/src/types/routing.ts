export type RoutingProfileId =
  | 'balanced'
  | 'first_byte'
  | 'high_tps'
  | 'cost_optimal'
  | 'high_availability'
  | 'cache_affinity_plus'
  | 'fixed_priority_plus'
  | 'custom';

export type RoutingRouteState =
  | 'eligible'
  | 'warming'
  | 'degraded'
  | 'excluded'
  | 'circuit_open';

export type RoutingMetricWindow = '1m' | '5m' | '15m' | '1h' | '24h' | '7d';

export type RoutingMetricSource = 'unknown' | 'exact' | 'window_fallback' | 'prior';

export type RoutingPriorSource =
  | 'unknown'
  | 'exact_route'
  | 'provider_model_format'
  | 'provider_model'
  | 'provider'
  | 'neutral';

export type RoutingRequestSizeBucket = 'unknown' | 'tiny' | 'small' | 'medium' | 'large' | 'huge';

export type RouteIdentity = {
  provider_id: string;
  key_id: string;
  endpoint_id: string;
  global_model_id: string;
  client_api_format: string;
  provider_api_format: string;
  is_stream: boolean;
};

export type RoutingRequestFeatures = {
  client_api_format: string;
  is_stream: boolean;
  input_token_estimate?: number | null;
  output_token_estimate?: number | null;
  request_size_bucket: RoutingRequestSizeBucket;
  required_capability?: string | null;
};

export type RoutingProfileWeights = {
  success: number;
  ttfb: number;
  latency: number;
  tps: number;
  cost: number;
  headroom: number;
  priority: number;
};

export type RoutingProfileLearningState = {
  admin_weights: RoutingProfileWeights;
  learned_weights?: RoutingProfileWeights | null;
  effective_weights: RoutingProfileWeights;
  reward_window: RoutingMetricWindow;
  sample_count: number;
  confidence: number;
  updated_at: string;
};

export type RoutingProfile = {
  id: RoutingProfileId;
  name: string;
  description: string;
  weights: RoutingProfileWeights;
  version: string;
  min_samples: number;
  exploration_k: number;
  conversion_penalty: number;
  stale_metric_penalty: number;
  affinity_bonus: number;
  prior_sample_cap: number;
  contextual_exploration_enabled: boolean;
  ema_alpha: number;
  ema_max_freshness_seconds: number;
  ema_recent_weight: number;
  ema_recent_cap: number;
  exploration_weight: number;
  exploration_cap: number;
  exploration_min_success_score: number;
  auto_tune_enabled: boolean;
  learning?: RoutingProfileLearningState | null;
};

export type RoutingProfilesResponse = {
  profiles: RoutingProfile[];
};

export type RoutingProfileUpsert = {
  weights?: RoutingProfileWeights;
  min_samples?: number;
  exploration_k?: number;
  conversion_penalty?: number;
  stale_metric_penalty?: number;
  affinity_bonus?: number;
  prior_sample_cap?: number;
  contextual_exploration_enabled?: boolean;
  ema_alpha?: number;
  ema_max_freshness_seconds?: number;
  ema_recent_weight?: number;
  ema_recent_cap?: number;
  exploration_weight?: number;
  exploration_cap?: number;
  exploration_min_success_score?: number;
  auto_tune_enabled?: boolean;
};

export type ScoreComponent = {
  code: string;
  label: string;
  raw_value?: number | null;
  normalized_score: number;
  weight: number;
  contribution: number;
};

export type RoutingMetricSnapshot = {
  request_count: number;
  success_count: number;
  failure_count: number;
  timeout_count: number;
  rate_limited_count: number;
  server_error_count: number;
  format_conversion_failure_count: number;
  usage_missing_count: number;
  stream_abnormal_end_count: number;
  schema_tool_call_failure_count: number;
  latency_avg_ms?: number | null;
  ttfb_avg_ms?: number | null;
  output_tps?: number | null;
  upstream_total_cost?: number | null;
  total_tokens: number;
  sample_count: number;
  rpm_used: number;
  rpm_limit?: number | null;
};

export type RouteScoreExplanation = {
  route: RouteIdentity;
  provider_name?: string | null;
  key_name?: string | null;
  key_preview?: string | null;
  endpoint_name?: string | null;
  rank: number;
  state: RoutingRouteState;
  final_score: number;
  metric_window: RoutingMetricWindow;
  selected_reason: string;
  components: ScoreComponent[];
  raw_metrics: RoutingMetricSnapshot;
  exclusion_reason?: string | null;
  metric_freshness_seconds: number;
  metric_source: RoutingMetricSource;
  prior_source: RoutingPriorSource;
  prior_sample_count: number;
  effective_sample_count: number;
  routing_context_key?: string | null;
  route_config_fingerprint?: string | null;
  price_config_fingerprint?: string | null;
  request_features: RoutingRequestFeatures;
};

export type RoutingRankingsQuery = {
  group_code: string;
  model: string;
  api_format: string;
  is_stream: boolean;
  window: RoutingMetricWindow;
  include_excluded: boolean;
  request_id_seed?: string;
};

export type RoutingRankingResponse = {
  profile: RoutingProfile;
  window: RoutingMetricWindow;
  selected?: RouteIdentity | null;
  request_id_seed: string;
  items: RouteScoreExplanation[];
};

export type RoutingDecisionResponse = {
  request_id: string;
  profile_id: RoutingProfileId;
  profile_version: string;
  selected?: RouteIdentity | null;
  candidates: RouteScoreExplanation[];
  created_at: string;
};
