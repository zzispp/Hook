import type {
  DisplayCurrency,
  ExchangeRateResponse,
  CurrencyDisplayResponse,
} from 'src/types/system-setting';

export type CurrencyDisplay = {
  currency: DisplayCurrency;
  usdCnyRate?: ExchangeRateResponse;
  unavailableLabel?: string;
};

const DISPLAY_DIGITS = 6;

export function currencyDisplayFromResponse(
  response: CurrencyDisplayResponse | undefined,
  unavailableLabel?: string
) {
  if (!response) return undefined;

  return {
    currency: response.currency,
    usdCnyRate: response.usd_cny_rate ?? undefined,
    unavailableLabel,
  } satisfies CurrencyDisplay;
}

export function formatMoney(value: number | null | undefined, display: CurrencyDisplay) {
  const amount = displayAmount(value, display);
  if (amount === null) return display.unavailableLabel ?? '-';
  return `${currencySymbol(display.currency)}${amount.toFixed(DISPLAY_DIGITS)}`;
}

export function formatMoneyCompact(value: number | null | undefined, display: CurrencyDisplay) {
  const amount = displayAmount(value, display);
  if (amount === null) return display.unavailableLabel ?? '-';
  return `${currencySymbol(display.currency)}${formatPrice(amount)}`;
}

export function currencySymbol(currency: DisplayCurrency) {
  return currency === 'CNY' ? '¥' : '$';
}

function displayAmount(value: number | null | undefined, display: CurrencyDisplay) {
  const amount = Number(value ?? 0);
  if (display.currency === 'CNY') {
    const rate = Number(display.usdCnyRate?.rate);
    if (!Number.isFinite(rate) || rate <= 0) return null;
    return amount * rate;
  }
  return amount;
}

function formatPrice(value: number) {
  if (value >= 0.01 || value === 0) return value.toFixed(2);
  if (value < 0.0001) return value.toExponential(2);
  return value.toFixed(4);
}
