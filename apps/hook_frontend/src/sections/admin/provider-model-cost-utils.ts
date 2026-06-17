import type { PricingTier, GlobalModelResponse } from 'src/types/model';
import type {
  ProviderApiKey,
  ProviderModelCost,
  ProviderModelBinding,
} from 'src/types/provider';

import { aetherCacheReadPrice, aetherCacheCreationPrice } from 'src/utils/model-pricing';

const CACHE_READ_TTL_MINUTES = 5;

export type TokenCostDraft = {
  input_price_per_million: string;
  output_price_per_million: string;
  cache_creation_price_per_million: string;
  cache_read_price_per_million: string;
};

export function bindingLabel(binding: ProviderModelBinding, models: GlobalModelResponse[]) {
  const model = findGlobalModel(models, binding.global_model_id);
  return model?.display_name || model?.name || binding.global_model_id;
}

export function bindingsAllowedForKey(key: ProviderApiKey, bindings: ProviderModelBinding[]) {
  if (key.allowed_model_ids.length === 0) return bindings;

  const allowedModelIds = new Set(key.allowed_model_ids);
  return bindings.filter((binding) => allowedModelIds.has(binding.global_model_id));
}

export function keyModelScopeLabel(key: ProviderApiKey, t: (key: string, options?: Record<string, unknown>) => string) {
  if (key.allowed_model_ids.length === 0) return t('providers.allModels');
  return t('providers.selectedModelCount', { count: key.allowed_model_ids.length });
}

export function findGlobalModel(models: GlobalModelResponse[], id: string) {
  return models.find((model) => model.id === id);
}

export function globalDefaultMode(binding: ProviderModelBinding, models: GlobalModelResponse[]) {
  const model = findGlobalModel(models, binding.global_model_id);
  return model?.default_tiered_pricing?.tiers?.length ? 'per_token' : 'per_request';
}

export function globalDefaultRequestPrice(binding: ProviderModelBinding, models: GlobalModelResponse[]) {
  return findGlobalModel(models, binding.global_model_id)?.default_price_per_request ?? null;
}

export function globalDefaultTokenDraft(binding: ProviderModelBinding, models: GlobalModelResponse[]) {
  return tokenDraftFromGlobal(binding, models, 1);
}

export function tokenDraftFromGlobal(
  binding: ProviderModelBinding,
  models: GlobalModelResponse[],
  multiplier: number
): TokenCostDraft {
  const tier = findGlobalModel(models, binding.global_model_id)?.default_tiered_pricing?.tiers?.[0];
  return tokenDraftFromTier(tier, multiplier);
}

export function effectiveTokenDraft(
  binding: ProviderModelBinding,
  models: GlobalModelResponse[],
  cost?: ProviderModelCost
): TokenCostDraft {
  if (cost?.cost_mode === 'per_token') {
    return {
      input_price_per_million: numberText(cost.input_price_per_million),
      output_price_per_million: numberText(cost.output_price_per_million),
      cache_creation_price_per_million: numberText(cost.cache_creation_price_per_million),
      cache_read_price_per_million: numberText(cost.cache_read_price_per_million),
    };
  }
  return tokenDraftFromGlobal(binding, models, 1);
}

export function effectiveRequestPrice(
  binding: ProviderModelBinding,
  models: GlobalModelResponse[],
  cost?: ProviderModelCost
) {
  if (cost?.cost_mode === 'per_request') return cost.price_per_request ?? null;
  return globalDefaultRequestPrice(binding, models);
}

export function numberText(value: number | null | undefined) {
  return value === null || value === undefined ? '' : String(value);
}

export function parseRequiredNumber(value: string) {
  const parsed = Number(value.trim());
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error('Invalid numeric field');
  }
  return parsed;
}

function tokenDraftFromTier(tier: PricingTier | undefined, multiplier: number): TokenCostDraft {
  const input = multiply(tier?.input_price_per_1m, multiplier);
  return {
    input_price_per_million: numberText(input),
    output_price_per_million: numberText(multiply(tier?.output_price_per_1m, multiplier)),
    cache_creation_price_per_million: numberText(cacheCreationPrice(tier, input, multiplier)),
    cache_read_price_per_million: numberText(cacheReadPrice(tier, input, multiplier)),
  };
}

function cacheCreationPrice(tier: PricingTier | undefined, input: number | undefined, multiplier: number) {
  const explicit = tier?.cache_creation_price_per_1m;
  return explicit === null || explicit === undefined
    ? aetherCacheCreationPrice(input)
    : multiply(explicit, multiplier);
}

function cacheReadPrice(tier: PricingTier | undefined, input: number | undefined, multiplier: number) {
  const explicit = tier?.cache_read_price_per_1m;
  if (explicit !== null && explicit !== undefined) return multiply(explicit, multiplier);
  const ttlPrice = tier?.cache_ttl_pricing?.find((item) => item.ttl_minutes === CACHE_READ_TTL_MINUTES)?.cache_read_price_per_1m;
  return ttlPrice === null || ttlPrice === undefined
    ? aetherCacheReadPrice(input)
    : multiply(ttlPrice, multiplier);
}

function multiply(value: number | null | undefined, multiplier: number) {
  if (value === null || value === undefined || !Number.isFinite(value)) return undefined;
  return Math.round(value * multiplier * 100000000) / 100000000;
}
