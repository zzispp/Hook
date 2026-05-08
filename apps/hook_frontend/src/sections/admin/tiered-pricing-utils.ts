import type { PricingTier, ModelsDevModelItem, TieredPricingConfig } from 'src/types/model';

import {
  CACHE_1H_TTL_MINUTES,
  aetherCacheReadPrice,
  aetherCacheCreationPrice,
  aetherCache1hCreationPrice,
} from 'src/utils/model-pricing';

// ----------------------------------------------------------------------

export const DEFAULT_PRICING: TieredPricingConfig = {
  tiers: [emptyTier(null)],
};

const CUSTOM_THRESHOLD_STEP = 200000;

export const THRESHOLD_OPTIONS = [
  { value: 64000, label: '64K' },
  { value: 128000, label: '128K' },
  { value: 200000, label: '200K' },
  { value: 500000, label: '500K' },
  { value: 1000000, label: '1M' },
] as const;

export function pricingFromModelsDev(item: ModelsDevModelItem): TieredPricingConfig {
  const input = item.inputPrice ?? 0;

  return {
    tiers: [
      withAetherCachePricing({
        up_to: null,
        input_price_per_1m: input,
        output_price_per_1m: item.outputPrice ?? 0,
        cache_creation_price_per_1m: item.cacheCreationPrice,
        cache_read_price_per_1m: item.cacheReadPrice,
        cache_ttl_pricing: [
          {
            ttl_minutes: CACHE_1H_TTL_MINUTES,
            cache_creation_price_per_1m: item.cache1hCreationPrice ?? 0,
          },
        ],
      }),
    ],
  };
}

export function normalizePricingConfig(pricing?: TieredPricingConfig | null): TieredPricingConfig {
  const tiers = pricing?.tiers?.length ? pricing.tiers : DEFAULT_PRICING.tiers;
  return { tiers: tiers.map((tier) => ({ ...tier, cache_ttl_pricing: cloneTtl(tier) })) };
}

export function finalPricingConfig(pricing: TieredPricingConfig): TieredPricingConfig {
  return { tiers: pricing.tiers.map(withAetherCachePricing) };
}

export function hasOneHourCachePricing(pricing: TieredPricingConfig) {
  return pricing.tiers.some((tier) =>
    tier.cache_ttl_pricing?.some((item) => item.ttl_minutes === CACHE_1H_TTL_MINUTES)
  );
}

export function updateTier(
  pricing: TieredPricingConfig,
  index: number,
  patch: Partial<PricingTier>
): TieredPricingConfig {
  return {
    tiers: pricing.tiers.map((tier, tierIndex) =>
      tierIndex === index ? { ...tier, ...patch } : tier
    ),
  };
}

export function addTier(pricing: TieredPricingConfig): TieredPricingConfig {
  const tiers = pricing.tiers.length ? pricing.tiers : DEFAULT_PRICING.tiers;
  const limit = nextThreshold(tiers);
  const updated = tiers.map((tier, index) =>
    index === tiers.length - 1 ? { ...tier, up_to: limit } : tier
  );

  return { tiers: [...updated, emptyTier(null)] };
}

export function removeTier(pricing: TieredPricingConfig, index: number): TieredPricingConfig {
  if (pricing.tiers.length <= 1) return pricing;

  const tiers = pricing.tiers.filter((_, tierIndex) => tierIndex !== index);
  return {
    tiers: tiers.map((tier, tierIndex) =>
      tierIndex === tiers.length - 1 ? { ...tier, up_to: null } : tier
    ),
  };
}

export function validatePricingConfig(pricing: TieredPricingConfig) {
  if (pricing.tiers.length === 0) return 'empty';
  if (pricing.tiers.at(-1)?.up_to !== null) return 'last-open';
  if (pricing.tiers.some(hasInvalidNumber)) return 'number';

  return hasInvalidOrder(pricing.tiers) ? 'order' : null;
}

export function formatTokens(tokens?: number | null) {
  if (tokens === null || tokens === undefined) return '';
  if (tokens >= 1000000) return `${tokens / 1000000}M`;
  if (tokens >= 1000) return `${tokens / 1000}K`;
  return String(tokens);
}

export function tierStartLabel(tiers: PricingTier[], index: number) {
  if (index === 0) return '0';
  return formatTokens(tiers[index - 1]?.up_to);
}

function withAetherCachePricing(tier: PricingTier): PricingTier {
  const input = tier.input_price_per_1m;
  const cache1h = oneHourCache(tier, input);

  return {
    ...tier,
    cache_creation_price_per_1m:
      tier.cache_creation_price_per_1m ?? aetherCacheCreationPrice(input),
    cache_read_price_per_1m: tier.cache_read_price_per_1m ?? aetherCacheReadPrice(input),
    cache_ttl_pricing: upsertOneHourCache(tier, cache1h),
  };
}

function oneHourCache(tier: PricingTier, input: number) {
  const existing = tier.cache_ttl_pricing?.find((item) => item.ttl_minutes === CACHE_1H_TTL_MINUTES);

  return {
    ttl_minutes: CACHE_1H_TTL_MINUTES,
    cache_creation_price_per_1m:
      existing?.cache_creation_price_per_1m ?? aetherCache1hCreationPrice(input) ?? 0,
    cache_read_price_per_1m: existing?.cache_read_price_per_1m,
  };
}

function cloneTtl(tier: PricingTier) {
  return tier.cache_ttl_pricing?.map((item) => ({ ...item })) ?? null;
}

function upsertOneHourCache(tier: PricingTier, cache1h: NonNullable<PricingTier['cache_ttl_pricing']>[number]) {
  const existing = tier.cache_ttl_pricing ?? [];
  const others = existing.filter((item) => item.ttl_minutes !== CACHE_1H_TTL_MINUTES);
  return [...others, cache1h];
}

function emptyTier(upTo: number | null): PricingTier {
  return {
    up_to: upTo,
    input_price_per_1m: 0,
    output_price_per_1m: 0,
  };
}

function nextThreshold(tiers: PricingTier[]) {
  const previous = tiers.at(-2)?.up_to ?? 0;
  return THRESHOLD_OPTIONS.find((option) => option.value > previous)?.value ?? previous + CUSTOM_THRESHOLD_STEP;
}

function hasInvalidNumber(tier: PricingTier) {
  return [tier.input_price_per_1m, tier.output_price_per_1m].some(
    (value) => !Number.isFinite(value) || value < 0
  );
}

function hasInvalidOrder(tiers: PricingTier[]) {
  let previous = 0;

  for (const tier of tiers.slice(0, -1)) {
    if (tier.up_to === null || tier.up_to <= previous) return true;
    previous = tier.up_to;
  }

  return false;
}
