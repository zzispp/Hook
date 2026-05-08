// ----------------------------------------------------------------------

export const CACHE_1H_TTL_MINUTES = 60;

const CACHE_CREATION_MULTIPLIER = 1.25;
const CACHE_READ_MULTIPLIER = 0.1;
const CACHE_1H_CREATION_MULTIPLIER = 2;
const PRICE_PRECISION = 4;

export function aetherCacheCreationPrice(inputPrice?: number | null) {
  return computedCachePrice(inputPrice, CACHE_CREATION_MULTIPLIER);
}

export function aetherCacheReadPrice(inputPrice?: number | null) {
  return computedCachePrice(inputPrice, CACHE_READ_MULTIPLIER);
}

export function aetherCache1hCreationPrice(inputPrice?: number | null) {
  return computedCachePrice(inputPrice, CACHE_1H_CREATION_MULTIPLIER);
}

function computedCachePrice(inputPrice: number | null | undefined, multiplier: number) {
  if (inputPrice === null || inputPrice === undefined) return undefined;
  if (!Number.isFinite(inputPrice)) return undefined;

  const factor = 10 ** PRICE_PRECISION;
  return Math.round(inputPrice * multiplier * factor) / factor;
}
