'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { AdminWallet, WalletSummary, WalletTransaction } from 'src/types/wallet';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { WALLET_TABLE_MIN_WIDTH } from './wallet-constants';
import {
  walletStatusLabel,
  formatSignedAmount,
  formatBalanceChange,
  formatWalletDateTime,
  formatBalanceBreakdown,
  walletTransactionColor,
  formatWalletLedgerSummary,
  walletTransactionReasonLabel,
  walletTransactionCategoryLabel,
} from './wallet-display';

type WalletLedgerTableProps = {
  wallet?: WalletSummary | AdminWallet;
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  items: WalletTransaction[];
  total: number;
  loadedCount: number;
  page: number;
  rowsPerPage: number;
  onOpen: (transaction: WalletTransaction) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function WalletLedgerTable({
  t,
  page,
  total,
  items,
  wallet,
  locale,
  loading,
  onOpen,
  loadedCount,
  rowsPerPage,
  onPageChange,
  onRowsPerPageChange,
}: WalletLedgerTableProps) {
  const head = tableHead(t);

  return (
    <>
      <WalletLedgerTableSummary t={t} shown={items.length} loaded={loadedCount} total={total} />
      <Scrollbar>
        <Table sx={{ minWidth: WALLET_TABLE_MIN_WIDTH }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {loading ? <LoadingRows t={t} head={head} rows={rowsPerPage} /> : null}
            {!loading ? renderRows(t, locale, items, wallet, onOpen) : null}
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

function WalletLedgerTableSummary({
  t,
  shown,
  loaded,
  total,
}: {
  t: TFunction<'admin'>;
  shown: number;
  loaded: number;
  total: number;
}) {
  return (
    <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
      {formatWalletLedgerSummary(t, shown, loaded, total)}
    </Typography>
  );
}

function renderRows(
  t: TFunction<'admin'>,
  locale: string,
  items: WalletTransaction[],
  wallet: WalletSummary | AdminWallet | undefined,
  onOpen: (transaction: WalletTransaction) => void
) {
  return items.map((transaction) => (
    <WalletLedgerRow
      key={transaction.id}
      t={t}
      wallet={wallet}
      locale={locale}
      transaction={transaction}
      onOpen={onOpen}
    />
  ));
}

function WalletLedgerRow({
  t,
  wallet,
  onOpen,
  locale,
  transaction,
}: {
  t: TFunction<'admin'>;
  wallet?: WalletSummary | AdminWallet;
  locale: string;
  transaction: WalletTransaction;
  onOpen: (transaction: WalletTransaction) => void;
}) {
  return (
    <TableRow hover sx={{ cursor: 'pointer' }} onClick={() => onOpen(transaction)}>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatWalletDateTime(transaction.created_at, locale)}
      </TableCell>
      <TableCell>
        <OwnerCell t={t} wallet={wallet} />
      </TableCell>
      <TableCell>
        <TransactionTypeCell t={t} transaction={transaction} />
      </TableCell>
      <AmountCell amount={transaction.amount} />
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        <BalanceChangeCell t={t} transaction={transaction} />
      </TableCell>
      <DescriptionCell t={t} description={transaction.description} />
      <DetailActionCell t={t} transaction={transaction} onOpen={onOpen} />
    </TableRow>
  );
}

function AmountCell({ amount }: { amount: number }) {
  const positive = amount >= 0;

  return (
    <TableCell sx={{ color: positive ? 'success.main' : 'error.main', fontWeight: 700 }}>
      {formatSignedAmount(amount)}
    </TableCell>
  );
}

function DescriptionCell({
  t,
  description,
}: {
  t: TFunction<'admin'>;
  description: string | null;
}) {
  return (
    <TableCell sx={{ color: 'text.secondary', maxWidth: 320 }}>
      <Typography variant="body2" noWrap>
        {description || t('wallet.emptyValue')}
      </Typography>
    </TableCell>
  );
}

function DetailActionCell({
  t,
  onOpen,
  transaction,
}: {
  t: TFunction<'admin'>;
  transaction: WalletTransaction;
  onOpen: (transaction: WalletTransaction) => void;
}) {
  return (
    <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
      <Button
        size="small"
        variant="outlined"
        startIcon={<Iconify icon="solar:eye-bold" />}
        sx={{ minWidth: 88, whiteSpace: 'nowrap' }}
        onClick={(event) => {
          event.stopPropagation();
          onOpen(transaction);
        }}
      >
        {t('wallet.actions.details')}
      </Button>
    </TableCell>
  );
}

function OwnerCell({ t, wallet }: { t: TFunction<'admin'>; wallet?: WalletSummary | AdminWallet }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="body2" sx={{ fontWeight: 600 }}>
        {walletOwnerName(t, wallet)}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('wallet.ownerSummary', { status: walletStatusLabel(t, wallet?.status) })}
      </Typography>
    </Stack>
  );
}

function walletOwnerName(t: TFunction<'admin'>, wallet?: WalletSummary | AdminWallet) {
  if (wallet && 'owner_name' in wallet) {
    return wallet.owner_name || wallet.owner_email || t('wallet.emptyValue');
  }

  return t('wallet.currentUser');
}

function TransactionTypeCell({
  t,
  transaction,
}: {
  t: TFunction<'admin'>;
  transaction: WalletTransaction;
}) {
  return (
    <Stack spacing={0.5} alignItems="flex-start">
      <Label color={walletTransactionColor(transaction.category)} variant="soft">
        {walletTransactionCategoryLabel(t, transaction.category)}
      </Label>
      <Typography variant="caption" color="text.secondary">
        {walletTransactionReasonLabel(t, transaction.reason_code)}
      </Typography>
    </Stack>
  );
}

function BalanceChangeCell({ t, transaction }: { t: TFunction<'admin'>; transaction: WalletTransaction }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
        {formatBalanceChange(transaction.balance_before, transaction.balance_after)}
      </Typography>
      <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
        {formatBalanceBreakdown(t, transaction)}
      </Typography>
    </Stack>
  );
}

function LoadingRows({
  t,
  rows,
  head,
}: {
  t: TFunction<'admin'>;
  rows: number;
  head: TableHeadCellProps[];
}) {
  return (
    <>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <TableRow key={rowIndex}>
          {head.map((cell) => (
            <TableCell key={cell.id} sx={{ color: 'text.disabled' }}>
              {t('common.loading')}
            </TableCell>
          ))}
        </TableRow>
      ))}
    </>
  );
}

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'created_at', label: t('wallet.table.time'), width: 170 },
    { id: 'owner', label: t('wallet.table.owner'), width: 180 },
    { id: 'category', label: t('wallet.table.type'), width: 160 },
    { id: 'amount', label: t('wallet.table.amount'), width: 130 },
    { id: 'balance', label: t('wallet.table.balanceChange'), width: 260 },
    { id: 'description', label: t('wallet.table.description') },
    { id: 'action', label: t('wallet.table.action'), width: 120, align: 'right' },
  ];
}
