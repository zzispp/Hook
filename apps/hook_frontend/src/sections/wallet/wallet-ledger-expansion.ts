'use client';

import type { WalletLedgerEntry } from 'src/types/wallet';
import type { WalletLedgerExpansionState } from './wallet-ledger-entries-table';

import { useState, useCallback } from 'react';

import { dailyUsageDate } from './wallet-ledger-entry-utils';
import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';

export type DailyUsageResource = {
  data?: {
    items: WalletLedgerExpansionState['items'];
    total: number;
  };
  isLoading: boolean;
  error?: Error;
};

export function useWalletLedgerExpansion() {
  const [entry, setEntry] = useState<WalletLedgerEntry | null>(null);
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(DEFAULT_WALLET_ROWS_PER_PAGE);

  const toggle = useCallback((nextEntry: WalletLedgerEntry) => {
    setEntry((current) => (current?.id === nextEntry.id ? null : nextEntry));
    setPage(0);
  }, []);

  const changePage = useCallback((nextEntry: WalletLedgerEntry, nextPage: number, nextPageSize: number) => {
    setEntry(nextEntry);
    setPage(nextPage);
    setPageSize(nextPageSize);
  }, []);

  const expansionState = useCallback(
    (resource: DailyUsageResource): WalletLedgerExpansionState => ({
      entryId: entry?.id ?? null,
      items: resource.data?.items ?? [],
      total: resource.data?.total ?? 0,
      loading: resource.isLoading,
      errorMessage: resource.error?.message,
    }),
    [entry?.id]
  );

  return {
    entry,
    date: entry ? dailyUsageDate(entry) : null,
    page,
    pageSize,
    toggle,
    changePage,
    expansionState,
  };
}
