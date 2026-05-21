'use client';

import type { TFunction } from 'i18next';
import type { WalletTransaction } from 'src/types/wallet';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Alert from '@mui/material/Alert';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import { useTable, TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { DEFAULT_WALLET_ROWS_PER_PAGE } from './wallet-constants';
import {
  formatSignedAmount,
  formatBalanceChange,
  formatWalletDateTime,
  formatBalanceBreakdown,
} from './wallet-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  items: WalletTransaction[];
  total: number;
  errorMessage?: string;
  onOpen: (transaction: WalletTransaction) => void;
  onPageChange: (page: number, pageSize: number) => void;
};

export function WalletLedgerDetailPanel({
  t,
  locale,
  loading,
  items,
  total,
  onOpen,
  errorMessage,
  onPageChange,
}: Props) {
  const table = useTable({ defaultRowsPerPage: DEFAULT_WALLET_ROWS_PER_PAGE });

  const handlePageChange = (event: unknown, page: number) => {
    table.onChangePage(event, page);
    onPageChange(page, table.rowsPerPage);
  };

  const handleRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement> = (event) => {
    table.onChangeRowsPerPage(event);
    onPageChange(0, parseInt(event.target.value, 10));
  };

  return (
    <Stack spacing={1.5} sx={{ p: 2, bgcolor: 'background.neutral' }}>
      {errorMessage ? <Alert severity="error">{errorMessage}</Alert> : null}
      <Scrollbar>
        <Table size="small" sx={{ minWidth: 760 }}>
          <TableHeadCustom headCells={detailHead(t)} />
          <TableBody>
            {loading ? <LoadingRows t={t} rows={table.rowsPerPage} /> : null}
            {!loading ? detailRows(t, locale, items, onOpen) : null}
            <TableNoData title={t('wallet.dailyUsage.emptyDetails')} notFound={!loading && items.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={handlePageChange}
        onRowsPerPageChange={handleRowsPerPageChange}
      />
    </Stack>
  );
}

function detailRows(
  t: TFunction<'admin'>,
  locale: string,
  items: WalletTransaction[],
  onOpen: (transaction: WalletTransaction) => void
) {
  return items.map((transaction) => (
    <TableRow key={transaction.id} hover sx={{ cursor: 'pointer' }} onClick={() => onOpen(transaction)}>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatWalletDateTime(transaction.created_at, locale)}</TableCell>
      <TableCell sx={{ color: transaction.amount >= 0 ? 'success.main' : 'error.main', fontWeight: 700 }}>
        {formatSignedAmount(transaction.amount)}
      </TableCell>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
            {formatBalanceChange(transaction.balance_before, transaction.balance_after)}
          </Typography>
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
            {formatBalanceBreakdown(t, transaction)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell sx={{ color: 'text.secondary' }}>
        <Typography variant="body2" noWrap sx={{ maxWidth: 260 }}>
          {transaction.description || t('wallet.emptyValue')}
        </Typography>
      </TableCell>
    </TableRow>
  ));
}

function LoadingRows({ t, rows }: { t: TFunction<'admin'>; rows: number }) {
  return Array.from({ length: rows }).map((_, rowIndex) => (
    <TableRow key={rowIndex}>
      {detailHead(t).map((cell) => (
        <TableCell key={cell.id} sx={{ color: 'text.disabled' }}>
          {t('common.loading')}
        </TableCell>
      ))}
    </TableRow>
  ));
}

function detailHead(t: TFunction<'admin'>) {
  return [
    { id: 'created_at', label: t('wallet.table.time'), width: 170 },
    { id: 'amount', label: t('wallet.table.amount'), width: 130 },
    { id: 'balance', label: t('wallet.table.balanceChange'), width: 260 },
    { id: 'description', label: t('wallet.table.description'), width: 260 },
  ];
}
