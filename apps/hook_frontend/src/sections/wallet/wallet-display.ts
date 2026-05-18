import type { TFunction } from 'i18next';
import type { LabelColor } from 'src/components/label';

import { formatMoney } from 'src/utils/currency-format';

import { MONEY_DECIMAL_PLACES } from './wallet-constants';

export function formatAccountingWalletMoney(value?: number | null) {
  return formatMoney(value);
}

export function formatWalletMoney(value?: number | null) {
  return formatMoney(value);
}

export function formatSignedAmount(value: number) {
  return `${value >= 0 ? '+' : '-'}${formatWalletMoney(Math.abs(value))}`;
}

export function formatWalletNumber(value?: number | null) {
  return Number(value ?? 0).toFixed(MONEY_DECIMAL_PLACES);
}

export function formatBalanceChange(before: number, after: number) {
  return `${formatWalletMoney(before)} -> ${formatWalletMoney(after)}`;
}

export function formatBalanceBreakdown(
  t: TFunction<'admin'>,
  transaction: {
    recharge_balance_before: number;
    recharge_balance_after: number;
    gift_balance_before: number;
    gift_balance_after: number;
  }
) {
  return t('wallet.balanceBreakdown', {
    rechargeBefore: formatWalletMoney(transaction.recharge_balance_before),
    rechargeAfter: formatWalletMoney(transaction.recharge_balance_after),
    giftBefore: formatWalletMoney(transaction.gift_balance_before),
    giftAfter: formatWalletMoney(transaction.gift_balance_after),
  });
}

export function formatWalletLedgerSummary(t: TFunction<'admin'>, shown: number, loaded: number, total: number) {
  return t('wallet.ledgerSummary', { shown, loaded, total });
}

export function formatAdminWalletSummary(t: TFunction<'admin'>, shown: number, total: number) {
  return t('adminWallets.walletSummary', { shown, total });
}

export function adminWalletOwner(wallet: { owner_name?: string; owner_email?: string }) {
  return wallet.owner_name || wallet.owner_email || '';
}

export function formatWalletDateTime(value: string, locale: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(date);
}

export function walletStatusLabel(t: TFunction<'admin'>, status?: string | null) {
  return labelFrom(t, 'wallet.statusLabels', status, 'wallet.unknown');
}

export function walletTransactionCategoryLabel(t: TFunction<'admin'>, category?: string | null) {
  return labelFrom(t, 'wallet.categoryLabels', category, 'wallet.unknown');
}

export function walletTransactionReasonLabel(t: TFunction<'admin'>, reasonCode?: string | null) {
  return labelFrom(t, 'wallet.reasonLabels', reasonCode, 'wallet.unknown');
}

export function walletLinkTypeLabel(t: TFunction<'admin'>, type?: string | null) {
  return labelFrom(t, 'wallet.linkTypeLabels', type, 'wallet.emptyValue');
}

export function walletTransactionColor(category?: string | null): LabelColor {
  if (category === 'refund' || category === 'consume') {
    return 'error';
  }

  if (category === 'recharge') {
    return 'success';
  }

  if (category === 'gift') {
    return 'info';
  }

  return 'warning';
}

function labelFrom(
  t: TFunction<'admin'>,
  prefix: string,
  value?: string | null,
  emptyKey = 'wallet.emptyValue'
) {
  if (!value) {
    return t(emptyKey);
  }

  const key = `${prefix}.${value}`;
  const translated = t(key);

  return translated === key ? value : translated;
}
