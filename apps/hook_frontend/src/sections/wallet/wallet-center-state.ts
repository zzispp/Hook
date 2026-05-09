'use client';

import type { TFunction } from 'i18next';
import type { WalletTransaction } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import { useWalletBalance, useWalletTransactions } from 'src/actions/wallet';

import { useTable } from 'src/components/table';

import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import {
  hasWalletFilters,
  walletFilterOptions,
  DEFAULT_WALLET_FILTERS,
  filterWalletTransactions,
} from './wallet-filters';

export function useWalletCenterState(t: TFunction<'admin'>) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const balance = useWalletBalance();
  const transactions = useWalletTransactions(table.page, table.rowsPerPage);
  const [filters, setFilters] = useState(DEFAULT_WALLET_FILTERS);
  const [currentTransaction, setCurrentTransaction] = useState<WalletTransaction | null>(null);

  const filteredItems = useMemo(
    () => filterWalletTransactions(transactions.items, filters, t),
    [filters, t, transactions.items]
  );
  const filterOptions = useMemo(() => walletFilterOptions(transactions.items, t), [t, transactions.items]);
  const wallet = balance.wallet ?? transactions.wallet;

  const changeFilters = useCallback(
    (nextFilters: typeof DEFAULT_WALLET_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  const refresh = useCallback(() => {
    balance.refresh();
    transactions.refresh();
  }, [balance, transactions]);

  return {
    table,
    wallet,
    filters,
    refresh,
    balance,
    transactions,
    filterOptions,
    filteredItems,
    changeFilters,
    currentTransaction,
    hasFilters: hasWalletFilters(filters),
    loading: balance.isLoading || transactions.isLoading,
    validating: balance.isValidating || transactions.isValidating,
    errorMessage: balance.error?.message || transactions.error?.message,
    setCurrentTransaction,
  };
}
