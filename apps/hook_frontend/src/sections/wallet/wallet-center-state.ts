'use client';

import type { TFunction } from 'i18next';
import type { WalletLedgerEntry, WalletTransaction } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import { useWalletBalance, useWalletLedgerEntries } from 'src/actions/wallet';

import { useTable } from 'src/components/table';

import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import {
  hasWalletFilters,
  DEFAULT_WALLET_FILTERS,
  walletEntryFilterOptions,
  toWalletLedgerEntryFilters,
} from './wallet-filters';

export function useWalletCenterState(t: TFunction<'admin'>) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const balance = useWalletBalance();
  const [filters, setFilters] = useState(DEFAULT_WALLET_FILTERS);
  const [currentTransaction, setCurrentTransaction] = useState<WalletTransaction | null>(null);
  const ledgerFilters = useMemo(() => toWalletLedgerEntryFilters(filters), [filters]);
  const entries = useWalletLedgerEntries(table.page, table.rowsPerPage, ledgerFilters);

  const items = useMemo((): WalletLedgerEntry[] => entries.data?.items ?? [], [entries.data?.items]);
  const filterOptions = useMemo(() => walletEntryFilterOptions(items, t), [items, t]);
  const wallet = balance.wallet ?? entries.data?.wallet;

  const changeFilters = useCallback(
    (nextFilters: typeof DEFAULT_WALLET_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  const refresh = useCallback(() => {
    balance.refresh();
    entries.refresh();
  }, [balance, entries]);

  return {
    table,
    wallet,
    filters,
    refresh,
    balance,
    entries,
    filterOptions,
    filteredItems: items,
    changeFilters,
    currentTransaction,
    hasFilters: hasWalletFilters(filters),
    loading: balance.isLoading || entries.isLoading,
    validating: balance.isValidating || entries.isValidating,
    errorMessage: balance.error?.message || entries.error?.message,
    setCurrentTransaction,
  };
}
