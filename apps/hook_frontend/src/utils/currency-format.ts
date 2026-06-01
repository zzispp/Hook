const DISPLAY_DIGITS = 6;
const COMPACT_DIGITS = 2;
const COMPACT_DETAIL_THRESHOLD = 0.01;
const USD_SYMBOL = '$';

export function formatMoney(value: number | null | undefined) {
  return `${USD_SYMBOL}${Number(value ?? 0).toFixed(DISPLAY_DIGITS)}`;
}

export function formatMoneyCompact(value: number | null | undefined) {
  return `${USD_SYMBOL}${formatPrice(Number(value ?? 0))}`;
}

function formatPrice(value: number) {
  const absolute = Math.abs(value);
  const maximumFractionDigits =
    absolute >= COMPACT_DETAIL_THRESHOLD || value === 0 ? COMPACT_DIGITS : DISPLAY_DIGITS;

  return new Intl.NumberFormat('en-US', {
    useGrouping: false,
    minimumFractionDigits: COMPACT_DIGITS,
    maximumFractionDigits,
  }).format(value);
}
