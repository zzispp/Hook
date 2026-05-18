export const ACCOUNTING_CURRENCY = 'USD';

export function labelWithAccountingCurrency(label: string) {
  return `${label} (${ACCOUNTING_CURRENCY})`;
}

export function accountingCurrencyLabel(label: string) {
  return labelWithAccountingCurrency(label);
}
