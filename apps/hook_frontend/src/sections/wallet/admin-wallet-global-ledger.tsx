'use client';

import type { TFunction } from 'i18next';
import type { AdminWalletLedgerTransaction } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';

import { useAdminWalletLedger } from 'src/actions/wallet';

import { useTable } from 'src/components/table';

import { WalletLedgerTable } from './wallet-ledger-table';
import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';
import {
  toAdminLedgerFilters,
  DEFAULT_ADMIN_LEDGER_FILTERS,
  AdminWalletLedgerFiltersToolbar,
} from './admin-wallet-ledger-filters';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
};

export function AdminWalletGlobalLedger({ t, locale }: Props) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_LEDGER_FILTERS);
  const ledgerFilters = useMemo(() => toAdminLedgerFilters(filters), [filters]);
  const ledger = useAdminWalletLedger(table.page, table.rowsPerPage, ledgerFilters);
  const [currentTransaction, setCurrentTransaction] = useState<AdminWalletLedgerTransaction | null>(null);
  const handleFiltersChange = useCallback(
    (nextFilters: typeof DEFAULT_ADMIN_LEDGER_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return (
    <>
      <Card>
        <AdminWalletLedgerFiltersToolbar
          t={t}
          loading={ledger.isLoading}
          filters={filters}
          onChange={handleFiltersChange}
          onRefresh={() => void ledger.refresh()}
        />
        <WalletLedgerTable
          t={t}
          locale={locale}
          loading={ledger.isLoading}
          items={ledger.data?.items ?? []}
          total={ledger.data?.total ?? 0}
          loadedCount={ledger.data?.items.length ?? 0}
          page={table.page}
          rowsPerPage={table.rowsPerPage}
          onOpen={(transaction) => setCurrentTransaction(transaction as AdminWalletLedgerTransaction)}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
        />
      </Card>
      <WalletTransactionDetailDialog
        t={t}
        locale={locale}
        transaction={currentTransaction}
        open={Boolean(currentTransaction)}
        onClose={() => setCurrentTransaction(null)}
      />
    </>
  );
}
