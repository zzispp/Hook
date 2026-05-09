import type { PricingTier, GlobalModelResponse, TieredPricingConfig } from 'src/types/model';

// ----------------------------------------------------------------------

const CONFIG_CAPABILITY_KEYS = [
  'vision',
  'function_calling',
  'streaming',
  'extended_thinking',
  'structured_output',
  'attachment',
  'image_generation',
] as const;

export const MODEL_DETAIL_CAPABILITIES = [
  { key: 'streaming', title: 'Streaming', descriptionKey: 'models.capabilityDescriptions.streaming' },
  {
    key: 'image_generation',
    title: 'Image Generation',
    descriptionKey: 'models.capabilityDescriptions.image_generation',
  },
  { key: 'vision', title: 'Vision', descriptionKey: 'models.capabilityDescriptions.vision' },
  { key: 'function_calling', title: 'Tool Use', descriptionKey: 'models.capabilityDescriptions.function_calling' },
  {
    key: 'extended_thinking',
    title: 'Extended Thinking',
    descriptionKey: 'models.capabilityDescriptions.extended_thinking',
  },
] as const;

export function filterCatalogItems(items: GlobalModelResponse[], query: string) {
  const normalized = query.trim().toLowerCase();
  if (!normalized) return items;

  return items.filter((item) => catalogItemMatches(item, normalized));
}

export function capabilityKeys(item: GlobalModelResponse) {
  const fromCapabilities = item.supported_capabilities ?? [];
  const fromConfig = CONFIG_CAPABILITY_KEYS.filter((key) => item.config?.[key] === true);
  return Array.from(new Set([...fromCapabilities, ...fromConfig]));
}

export function hasCapability(item: GlobalModelResponse, key: string) {
  if (key === 'streaming') return configBool(item, key, true);
  if (configBool(item, key, false)) return true;
  return item.supported_capabilities?.includes(key) ?? false;
}

export function priceSummary(item: GlobalModelResponse) {
  return `${formatPrice(firstTier(item)?.input_price_per_1m)} / ${formatPrice(firstTier(item)?.output_price_per_1m)}`;
}

export function tierCount(pricing?: TieredPricingConfig | null) {
  return pricing?.tiers?.length ?? 0;
}

export function firstTierPrice(
  pricing: TieredPricingConfig | undefined | null,
  key: keyof Pick<
    PricingTier,
    | 'input_price_per_1m'
    | 'output_price_per_1m'
    | 'cache_creation_price_per_1m'
    | 'cache_read_price_per_1m'
  >
) {
  return formatPrice(pricing?.tiers?.[0]?.[key]);
}

export function firstOneHourCachePrice(pricing?: TieredPricingConfig | null) {
  return oneHourCachePrice(pricing?.tiers?.[0]);
}

export function oneHourCachePrice(tier?: PricingTier | null) {
  const price = tier?.cache_ttl_pricing?.find((item) => item.ttl_minutes === 60);
  return formatPrice(price?.cache_creation_price_per_1m);
}

export function requestPrice(value?: number | null) {
  if (!value || value <= 0) return null;
  return `${formatPrice(value)}/次`;
}

export function formatUsageCount(value?: number | null) {
  return value?.toLocaleString() ?? '0';
}

export function formatPrice(value?: number | null) {
  return value === null || value === undefined ? '-' : `$${formatNumber(value)}`;
}

function catalogItemMatches(item: GlobalModelResponse, query: string) {
  return [item.name, item.display_name, description(item)]
    .join(' ')
    .toLowerCase()
    .includes(query);
}

function description(item: GlobalModelResponse) {
  const value = item.config?.description;
  return typeof value === 'string' ? value : '';
}

function firstTier(item: GlobalModelResponse) {
  return item.default_tiered_pricing?.tiers?.[0];
}

function configBool(item: GlobalModelResponse, key: string, defaultValue: boolean) {
  const value = item.config?.[key];
  return typeof value === 'boolean' ? value : defaultValue;
}

function formatNumber(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}
