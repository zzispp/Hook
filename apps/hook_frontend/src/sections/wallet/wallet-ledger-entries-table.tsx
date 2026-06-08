'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { AdminWallet, WalletSummary, WalletLedgerEntry, WalletTransaction } from 'src/types/wallet';

import Table from '@mui/material/Table';
import Collapse from '@mui/material/Collapse';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableHeadCustom,
  TablePaginationCustom,
  withStickyActionHeadCell,
} from 'src/components/table';

import { WALLET_TABLE_MIN_WIDTH } from './wallet-constants';
import { formatWalletLedgerSummary } from './wallet-display';
import { WalletLedgerEntryRow } from './wallet-ledger-entry-row';
import { WalletLedgerDetailPanel } from './wallet-ledger-detail-panel';

export type WalletLedgerExpansionState = {
  entryId: string | null;
  items: WalletTransaction[];
  total: number;
  loading: boolean;
  errorMessage?: string;
};

type Props = {
  wallet?: WalletSummary | AdminWallet;
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  items: WalletLedgerEntry[];
  total: number;
  loadedCount: number;
  page: number;
  rowsPerPage: number;
  expansion: WalletLedgerExpansionState;
  onOpen: (transaction: WalletTransaction) => void;
  onToggleDailyUsage: (entry: WalletLedgerEntry) => void;
  onDailyUsagePageChange: (entry: WalletLedgerEntry, page: number, pageSize: number) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function WalletLedgerEntriesTable({
  t,
  page,
  total,
  items,
  wallet,
  locale,
  loading,
  onOpen,
  expansion,
  loadedCount,
  rowsPerPage,
  onPageChange,
  onToggleDailyUsage,
  onRowsPerPageChange,
  onDailyUsagePageChange,
}: Props) {
  const head = tableHead(t);

  return (
    <>
      <WalletLedgerTableSummary t={t} shown={items.length} loaded={loadedCount} total={total} />
      <Scrollbar>
        <Table sx={{ minWidth: WALLET_TABLE_MIN_WIDTH }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {loading ? <LoadingRows t={t} head={head} rows={rowsPerPage} /> : null}
            {!loading
              ? items.map((entry) => (
                  <WalletLedgerEntryRows
                    key={entry.id}
                    t={t}
                    entry={entry}
                    wallet={wallet}
                    locale={locale}
                    onOpen={onOpen}
                    expansion={expansion}
                    onToggleDailyUsage={onToggleDailyUsage}
                    onDailyUsagePageChange={onDailyUsagePageChange}
                  />
                ))
              : null}
            <TableNoData title={t('wallet.emptyLedger')} notFound={!loading && items.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={page}
        count={total}
        rowsPerPage={rowsPerPage}
        onPageChange={onPageChange}
        onRowsPerPageChange={onRowsPerPageChange}
      />
    </>
  );
}

function WalletLedgerEntryRows({
  t,
  entry,
  wallet,
  onOpen,
  locale,
  expansion,
  onToggleDailyUsage,
  onDailyUsagePageChange,
}: {
  t: TFunction<'admin'>;
  entry: WalletLedgerEntry;
  wallet?: WalletSummary | AdminWallet;
  locale: string;
  expansion: WalletLedgerExpansionState;
  onOpen: (transaction: WalletTransaction) => void;
  onToggleDailyUsage: (entry: WalletLedgerEntry) => void;
  onDailyUsagePageChange: (entry: WalletLedgerEntry, page: number, pageSize: number) => void;
}) {
  const expanded = expansion.entryId === entry.id;

  return (
    <>
      <WalletLedgerEntryRow
        t={t}
        entry={entry}
        wallet={wallet}
        locale={locale}
        expanded={expanded}
        onOpen={onOpen}
        onToggleDailyUsage={onToggleDailyUsage}
      />
      <TableRow>
        <TableCell colSpan={7} sx={{ p: 0, border: 0 }}>
          <Collapse in={expanded} timeout="auto" unmountOnExit>
            <WalletLedgerDetailPanel
              t={t}
              locale={locale}
              items={expansion.items}
              total={expansion.total}
              loading={expansion.loading}
              errorMessage={expansion.errorMessage}
              onOpen={onOpen}
              onPageChange={(page, pageSize) => onDailyUsagePageChange(entry, page, pageSize)}
            />
          </Collapse>
        </TableCell>
      </TableRow>
    </>
  );
}

function WalletLedgerTableSummary({ t, shown, loaded, total }: { t: TFunction<'admin'>; shown: number; loaded: number; total: number }) {
  return (
    <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
      {formatWalletLedgerSummary(t, shown, loaded, total)}
    </Typography>
  );
}

function LoadingRows({ t, rows, head }: { t: TFunction<'admin'>; rows: number; head: TableHeadCellProps[] }) {
  return Array.from({ length: rows }).map((_, rowIndex) => (
    <TableRow key={rowIndex}>
      {head.map((cell) => (
        <TableCell key={cell.id} sx={{ color: 'text.disabled' }}>
          {t('common.loading')}
        </TableCell>
      ))}
    </TableRow>
  ));
}

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'created_at', label: t('wallet.table.time'), width: 170 },
    { id: 'owner', label: t('wallet.table.owner'), width: 180 },
    { id: 'category', label: t('wallet.table.type'), width: 160 },
    { id: 'amount', label: t('wallet.table.amount'), width: 130 },
    { id: 'balance', label: t('wallet.table.balanceChange'), width: 260 },
    { id: 'description', label: t('wallet.table.description'), width: 240 },
    withStickyActionHeadCell({
      id: 'action',
      label: t('wallet.table.action'),
      width: 120,
      align: 'left',
    }),
  ];
}
