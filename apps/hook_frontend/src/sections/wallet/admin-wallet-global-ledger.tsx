'use client';

import type { TFunction } from 'i18next';
import type { WalletTransaction, AdminWalletLedgerEntry } from 'src/types/wallet';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';

import {
  useAdminWalletLedgerEntries,
  useAdminWalletDailyModelUsage,
  useAdminWalletConsumptionSummary,
} from 'src/actions/wallet';

import { useTable } from 'src/components/table';

import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import { useWalletLedgerExpansion } from './wallet-ledger-expansion';
import { WalletLedgerEntriesTable } from './wallet-ledger-entries-table';
import { WalletTransactionDetailDialog } from './wallet-transaction-detail-dialog';
import { AdminWalletConsumptionSummaryTable } from './admin-wallet-consumption-summary-table';
import {
  toAdminLedgerFilters,
  DEFAULT_ADMIN_LEDGER_FILTERS,
  AdminWalletLedgerFiltersToolbar,
  toAdminConsumptionSummaryFilters,
} from './admin-wallet-ledger-filters';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
};

export function AdminWalletGlobalLedger({ t, locale }: Props) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE, defaultOrderBy: 'created_at' });
  const [filters, setFilters] = useState(DEFAULT_ADMIN_LEDGER_FILTERS);
  const ledgerFilters = useMemo(() => toAdminLedgerFilters(filters), [filters]);
  const summaryFilters = useMemo(() => toAdminConsumptionSummaryFilters(filters), [filters]);
  const ledger = useAdminWalletLedgerEntries(table.page, table.rowsPerPage, ledgerFilters, !filters.aggregateConsumption);
  const summary = useAdminWalletConsumptionSummary(table.page, table.rowsPerPage, summaryFilters, filters.aggregateConsumption);
  const [currentTransaction, setCurrentTransaction] = useState<WalletTransaction | null>(null);
  const expansion = useWalletLedgerExpansion();
  const walletId = expandedWalletId(ledger.data?.items ?? [], expansion.entry?.id ?? null);
  const detail = useAdminWalletDailyModelUsage(walletId, expansion.date, expansion.page, expansion.pageSize);
  const loading = filters.aggregateConsumption ? summary.isLoading : ledger.isLoading;
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
          loading={loading}
          filters={filters}
          onChange={handleFiltersChange}
          onRefresh={() => void (filters.aggregateConsumption ? summary.refresh() : ledger.refresh())}
        />
        {filters.aggregateConsumption ? (
          <AdminWalletConsumptionSummaryTable
            t={t}
            locale={locale}
            loading={summary.isLoading}
            items={summary.data?.items ?? []}
            total={summary.data?.total ?? 0}
            page={table.page}
            rowsPerPage={table.rowsPerPage}
            onPageChange={table.onChangePage}
            onRowsPerPageChange={table.onChangeRowsPerPage}
          />
        ) : (
          <WalletLedgerEntriesTable
            t={t}
            locale={locale}
            loading={ledger.isLoading}
            items={ledger.data?.items ?? []}
            total={ledger.data?.total ?? 0}
            loadedCount={ledger.data?.items.length ?? 0}
            page={table.page}
            rowsPerPage={table.rowsPerPage}
            expansion={expansion.expansionState(detail)}
            onOpen={setCurrentTransaction}
            onToggleDailyUsage={expansion.toggle}
            onDailyUsagePageChange={expansion.changePage}
            onPageChange={table.onChangePage}
            onRowsPerPageChange={table.onChangeRowsPerPage}
          />
        )}
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

function expandedWalletId(items: AdminWalletLedgerEntry[], entryId: string | null) {
  return items.find((item) => item.id === entryId)?.wallet_id ?? null;
}
