import type { TFunction } from 'i18next';
import type { WalletTransaction } from 'src/types/wallet';

import { ALL_FILTER_VALUE } from './wallet-constants';
import {
  walletLinkTypeLabel,
  walletTransactionReasonLabel,
  walletTransactionCategoryLabel,
} from './wallet-display';

export type WalletLedgerFilterState = {
  search: string;
  category: string;
  reason: string;
  direction: string;
  balanceType: string;
  linkType: string;
};

export type WalletFilterOption = {
  value: string;
  label: string;
};

export const DEFAULT_WALLET_FILTERS: WalletLedgerFilterState = {
  search: '',
  category: ALL_FILTER_VALUE,
  reason: ALL_FILTER_VALUE,
  direction: ALL_FILTER_VALUE,
  balanceType: ALL_FILTER_VALUE,
  linkType: ALL_FILTER_VALUE,
};

export const CATEGORY_VALUES = [
  ALL_FILTER_VALUE,
  'recharge',
  'gift',
  'adjust',
  'refund',
  'consume',
] as const;

export const DIRECTION_VALUES = [ALL_FILTER_VALUE, 'income', 'expense'] as const;

export const BALANCE_TYPE_VALUES = [ALL_FILTER_VALUE, 'recharge', 'gift'] as const;

export function filterWalletTransactions(
  items: WalletTransaction[],
  filters: WalletLedgerFilterState,
  t: TFunction<'admin'>
) {
  return items.filter((transaction) => matchesWalletFilters(transaction, filters, t));
}

export function walletFilterOptions(items: WalletTransaction[], t: TFunction<'admin'>) {
  return {
    reasons: buildOptions(items.map((item) => item.reason_code), (value) => walletTransactionReasonLabel(t, value)),
    linkTypes: buildOptions(items.map((item) => item.link_type).filter(Boolean), (value) => walletLinkTypeLabel(t, value)),
  };
}

export function walletStaticFilterOptions(t: TFunction<'admin'>) {
  return {
    categories: CATEGORY_VALUES.map((value) => ({ value, label: categoryOptionLabel(t, value) })),
    directions: DIRECTION_VALUES.map((value) => ({ value, label: directionOptionLabel(t, value) })),
    balanceTypes: BALANCE_TYPE_VALUES.map((value) => ({ value, label: balanceTypeOptionLabel(t, value) })),
  };
}

export function hasWalletFilters(filters: WalletLedgerFilterState) {
  return Object.entries(filters).some(([key, value]) => {
    if (key === 'search') {
      return value.trim() !== '';
    }

    return value !== ALL_FILTER_VALUE;
  });
}

function matchesWalletFilters(
  transaction: WalletTransaction,
  filters: WalletLedgerFilterState,
  t: TFunction<'admin'>
) {
  return (
    matchesSearch(transaction, filters.search, t) &&
    matchesExact(transaction.category, filters.category) &&
    matchesExact(transaction.reason_code, filters.reason) &&
    matchesExact(transaction.link_type, filters.linkType) &&
    matchesDirection(transaction.amount, filters.direction) &&
    matchesBalanceType(transaction, filters.balanceType)
  );
}

function matchesSearch(transaction: WalletTransaction, search: string, t: TFunction<'admin'>) {
  const keyword = search.trim().toLowerCase();

  if (!keyword) {
    return true;
  }

  return searchableText(transaction, t).includes(keyword);
}

function matchesExact(value: string | null, filter: string) {
  return filter === ALL_FILTER_VALUE || value === filter;
}

function matchesDirection(amount: number, direction: string) {
  if (direction === ALL_FILTER_VALUE) {
    return true;
  }

  return direction === 'income' ? amount >= 0 : amount < 0;
}

function matchesBalanceType(transaction: WalletTransaction, balanceType: string) {
  if (balanceType === ALL_FILTER_VALUE) {
    return true;
  }

  const before = balanceType === 'recharge' ? transaction.recharge_balance_before : transaction.gift_balance_before;
  const after = balanceType === 'recharge' ? transaction.recharge_balance_after : transaction.gift_balance_after;

  return before !== after;
}

function searchableText(transaction: WalletTransaction, t: TFunction<'admin'>) {
  return [
    transaction.id,
    transaction.category,
    transaction.reason_code,
    transaction.link_type,
    transaction.link_id,
    transaction.operator_id,
    transaction.description,
    walletTransactionCategoryLabel(t, transaction.category),
    walletTransactionReasonLabel(t, transaction.reason_code),
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
}

function categoryOptionLabel(t: TFunction<'admin'>, value: string) {
  return value === ALL_FILTER_VALUE ? t('wallet.filters.allCategories') : walletTransactionCategoryLabel(t, value);
}

function directionOptionLabel(t: TFunction<'admin'>, value: string) {
  if (value === ALL_FILTER_VALUE) {
    return t('wallet.filters.allDirections');
  }

  return t(`wallet.directionLabels.${value}`);
}

function balanceTypeOptionLabel(t: TFunction<'admin'>, value: string) {
  if (value === ALL_FILTER_VALUE) {
    return t('wallet.filters.allBalances');
  }

  return t(`wallet.balanceTypeLabels.${value}`);
}

function buildOptions(values: (string | null)[], labeler: (value?: string | null) => string) {
  return Array.from(new Set(values))
    .filter((value): value is string => Boolean(value))
    .map((value) => ({ value, label: labeler(value) }))
    .sort((left, right) => left.label.localeCompare(right.label, 'zh-CN'));
}
