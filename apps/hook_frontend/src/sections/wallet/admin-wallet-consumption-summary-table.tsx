'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { AdminWalletConsumptionSummaryItem } from 'src/types/wallet';

import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { WALLET_TABLE_MIN_WIDTH } from './wallet-constants';
import { formatWalletMoney, walletStatusLabel, formatWalletDateTime } from './wallet-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  loading: boolean;
  items: AdminWalletConsumptionSummaryItem[];
  total: number;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function AdminWalletConsumptionSummaryTable({
  t,
  page,
  total,
  items,
  locale,
  loading,
  rowsPerPage,
  onPageChange,
  onRowsPerPageChange,
}: Props) {
  const head = tableHead(t);

  return (
    <>
      <SummaryText t={t} shown={items.length} total={total} />
      <SummaryTable t={t} locale={locale} loading={loading} items={items} rowsPerPage={rowsPerPage} head={head} />
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

function SummaryText({ t, shown, total }: { t: TFunction<'admin'>; shown: number; total: number }) {
  return (
    <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
      {t('wallet.consumptionSummary.summary', { shown, total })}
    </Typography>
  );
}

function SummaryTable({
  t,
  head,
  items,
  locale,
  loading,
  rowsPerPage,
}: {
  t: TFunction<'admin'>;
  head: TableHeadCellProps[];
  items: AdminWalletConsumptionSummaryItem[];
  locale: string;
  loading: boolean;
  rowsPerPage: number;
}) {
  return (
    <Scrollbar>
      <Table sx={{ minWidth: WALLET_TABLE_MIN_WIDTH }}>
        <TableHeadCustom headCells={head} />
        <TableBody>
          {loading ? <LoadingRows t={t} head={head} rows={rowsPerPage} /> : null}
          {!loading ? items.map((item) => <SummaryRow key={item.wallet_id} t={t} item={item} locale={locale} />) : null}
          <TableNoData title={t('wallet.consumptionSummary.empty')} notFound={!loading && items.length === 0} />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function SummaryRow({ t, item, locale }: { t: TFunction<'admin'>; item: AdminWalletConsumptionSummaryItem; locale: string }) {
  return (
    <TableRow hover>
      <OwnerCell t={t} item={item} />
      <TableCell sx={{ color: 'error.main', fontWeight: 700 }}>
        {formatWalletMoney(item.consumed_amount)}
      </TableCell>
      <TableCell>{item.transaction_count}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatWalletDateTime(item.first_created_at, locale)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatWalletDateTime(item.last_created_at, locale)}</TableCell>
    </TableRow>
  );
}

function OwnerCell({ t, item }: { t: TFunction<'admin'>; item: AdminWalletConsumptionSummaryItem }) {
  return (
    <TableCell>
      <Typography variant="body2" sx={{ fontWeight: 600 }}>
        {item.owner_name || item.user_id}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {t('wallet.ownerSummary', {
          type: t(`wallet.ownerTypes.${item.owner_type}`),
          status: walletStatusLabel(t, item.wallet_status),
        })}
      </Typography>
    </TableCell>
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
    { id: 'owner', label: t('wallet.consumptionSummary.owner'), width: 220 },
    { id: 'consumed_amount', label: t('wallet.consumptionSummary.amount'), width: 150 },
    { id: 'transaction_count', label: t('wallet.consumptionSummary.count'), width: 130 },
    { id: 'first_created_at', label: t('wallet.consumptionSummary.firstConsumedAt'), width: 180 },
    { id: 'last_created_at', label: t('wallet.consumptionSummary.lastConsumedAt'), width: 180 },
  ];
}
