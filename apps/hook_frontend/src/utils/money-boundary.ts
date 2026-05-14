import type { DisplayCurrency } from 'src/types/system-setting';

export const ACCOUNTING_CURRENCY: DisplayCurrency = 'USD';
export const DEFAULT_WALLET_CURRENCY = 'CNY';

export function labelWithCurrency(label: string, currency?: string | null) {
  return currency ? `${label} (${currency})` : label;
}

export function accountingCurrencyLabel(label: string) {
  return labelWithCurrency(label, ACCOUNTING_CURRENCY);
}
