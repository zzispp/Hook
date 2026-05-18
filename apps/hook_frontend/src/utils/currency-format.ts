const DISPLAY_DIGITS = 6;
const USD_SYMBOL = '$';

export function formatMoney(value: number | null | undefined) {
  return `${USD_SYMBOL}${Number(value ?? 0).toFixed(DISPLAY_DIGITS)}`;
}

export function formatMoneyCompact(value: number | null | undefined) {
  return `${USD_SYMBOL}${formatPrice(Number(value ?? 0))}`;
}

function formatPrice(value: number) {
  if (value >= 0.01 || value === 0) return value.toFixed(2);
  if (value < 0.0001) return value.toExponential(2);
  return value.toFixed(4);
}
