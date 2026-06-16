import type { RoutingProfileId } from './routing';

// ----------------------------------------------------------------------

export type CacheTTLPricing = {
  ttl_minutes: number;
  cache_creation_price_per_1m: number;
  cache_read_price_per_1m?: number | null;
};

export type PricingTier = {
  up_to: number | null;
  input_price_per_1m: number;
  output_price_per_1m: number;
  cache_creation_price_per_1m?: number | null;
  cache_read_price_per_1m?: number | null;
  cache_ttl_pricing?: CacheTTLPricing[] | null;
};

export type TieredPricingConfig = {
  tiers: PricingTier[];
};

export type ModelCapabilities = {
  supports_vision: boolean;
  supports_function_calling: boolean;
  supports_streaming: boolean;
};

export type ModelPriceRange = {
  min_input: number | null;
  max_input: number | null;
  min_output: number | null;
  max_output: number | null;
};

export type ModelCatalogProviderPriceRange = {
  min: number | null;
  max: number | null;
};

export type ModelCatalogProviderDetail = {
  provider_id: string;
  provider_name: string;
  model_id?: string | null;
  target_model: string;
  is_active: boolean;
  provider_is_active: boolean;
  binding_is_active: boolean;
  configured_cost_count: number;
  input_price_per_1m?: number | null;
  input_price_range: ModelCatalogProviderPriceRange;
  output_price_per_1m?: number | null;
  output_price_range: ModelCatalogProviderPriceRange;
  cache_creation_price_per_1m?: number | null;
  cache_creation_price_range: ModelCatalogProviderPriceRange;
  cache_read_price_per_1m?: number | null;
  cache_read_price_range: ModelCatalogProviderPriceRange;
  cache_1h_creation_price_per_1m?: number | null;
  price_per_request?: number | null;
  price_per_request_range: ModelCatalogProviderPriceRange;
  effective_tiered_pricing?: TieredPricingConfig | null;
  tier_count: number;
  supports_vision?: boolean | null;
  supports_function_calling?: boolean | null;
  supports_streaming?: boolean | null;
};

export type ModelCatalogItem = {
  global_model_id: string;
  global_model_name: string;
  display_name: string;
  description?: string | null;
  providers: ModelCatalogProviderDetail[];
  price_range: ModelPriceRange;
  total_providers: number;
  capabilities: ModelCapabilities;
};

export type ModelCatalogResponse = {
  models: ModelCatalogItem[];
  total: number;
};

export type GlobalModelCreate = {
  name: string;
  display_name: string;
  default_price_per_request?: number | null;
  default_tiered_pricing: TieredPricingConfig;
  supported_capabilities?: string[] | null;
  config?: Record<string, unknown> | null;
  routing_profile_id?: RoutingProfileId | null;
  is_active?: boolean;
};

export type GlobalModelUpdate = {
  display_name?: string;
  is_active?: boolean;
  default_price_per_request?: number | null;
  default_tiered_pricing?: TieredPricingConfig | null;
  supported_capabilities?: string[] | null;
  config?: Record<string, unknown> | null;
  routing_profile_id?: RoutingProfileId | null;
};

export type GlobalModelResponse = {
  id: string;
  name: string;
  display_name: string;
  is_active: boolean;
  default_price_per_request?: number | null;
  default_tiered_pricing: TieredPricingConfig;
  supported_capabilities?: string[] | null;
  config?: Record<string, unknown> | null;
  routing_profile_id?: RoutingProfileId | null;
  provider_count?: number | null;
  active_provider_count?: number | null;
  usage_count?: number | null;
  created_at: string;
  updated_at?: string | null;
};

export type GlobalModelWithStats = GlobalModelResponse & {
  total_models: number;
  total_providers: number;
  price_range: ModelPriceRange;
};

export type GlobalModelListResponse = {
  models: GlobalModelResponse[];
  total: number;
};

export type BatchDeleteGlobalModelsResponse = {
  success_count: number;
  failed: { id: string; error: string }[];
};

export type GlobalModelProvidersResponse = {
  providers: ModelCatalogProviderDetail[];
  total: number;
};

export type ModelsDevCost = {
  input?: number;
  output?: number;
  reasoning?: number;
  cache_read?: number;
  cache_write?: number;
};

export type ModelsDevLimit = {
  context?: number;
  output?: number;
};

export type ModelsDevModel = {
  id?: string;
  name?: string;
  family?: string;
  reasoning?: boolean;
  tool_call?: boolean;
  structured_output?: boolean;
  temperature?: boolean;
  attachment?: boolean;
  knowledge?: string;
  release_date?: string;
  last_updated?: string;
  input?: string[];
  output?: string[];
  modalities?: {
    input?: string[];
    output?: string[];
  };
  open_weights?: boolean;
  cost?: ModelsDevCost;
  limit?: ModelsDevLimit;
  deprecated?: boolean;
};

export type ModelsDevProvider = {
  id?: string;
  env?: string[];
  npm?: string;
  api?: string;
  name?: string;
  doc?: string;
  models?: Record<string, ModelsDevModel>;
  official?: boolean;
};

export type ModelsDevData = Record<string, ModelsDevProvider>;

export type ModelsDevModelItem = {
  providerId: string;
  providerName: string;
  modelId: string;
  modelName: string;
  family?: string;
  inputPrice?: number;
  outputPrice?: number;
  cacheCreationPrice?: number;
  cacheReadPrice?: number;
  cache1hCreationPrice?: number;
  contextLimit?: number;
  outputLimit?: number;
  supportsVision: boolean;
  supportsToolCall: boolean;
  supportsReasoning: boolean;
  supportsStructuredOutput: boolean;
  supportsTemperature: boolean;
  supportsAttachment: boolean;
  openWeights: boolean;
  deprecated: boolean;
  official: boolean;
  knowledgeCutoff?: string;
  releaseDate?: string;
  inputModalities: string[];
  outputModalities: string[];
};
