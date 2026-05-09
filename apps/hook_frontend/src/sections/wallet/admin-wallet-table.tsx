'use client';

import type { TFunction } from 'i18next';
import type { AdminWallet } from 'src/types/wallet';
import type { TableHeadCellProps } from 'src/components/table';

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

import {
  adminWalletOwner,
  walletStatusLabel,
  formatWalletMoney,
  formatWalletDateTime,
  formatAdminWalletSummary,
} from './wallet-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rows: AdminWallet[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onOpenLedger: (wallet: AdminWallet) => void;
  onOpenAdjust: (wallet: AdminWallet) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function AdminWalletTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {formatAdminWalletSummary(props.t, props.rows.length, props.total)}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1120 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading ? props.rows.map((row) => <AdminWalletRow key={row.id} row={row} {...props} />) : null}
            <TableNoData title={props.t('common.noData')} notFound={!props.loading && props.rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={props.page}
        count={props.total}
        rowsPerPage={props.rowsPerPage}
        onPageChange={props.onPageChange}
        onRowsPerPageChange={props.onRowsPerPageChange}
      />
    </>
  );
}

function AdminWalletRow({
  t,
  row,
  locale,
  onOpenLedger,
  onOpenAdjust,
}: Pick<Props, 't' | 'locale' | 'onOpenLedger' | 'onOpenAdjust'> & { row: AdminWallet }) {
  return (
    <TableRow hover>
      <TableCell>
        <OwnerCell t={t} row={row} />
      </TableCell>
      <TableCell>{formatWalletMoney(row.balance, row.currency)}</TableCell>
      <TableCell>{formatWalletMoney(row.recharge_balance, row.currency)}</TableCell>
      <TableCell>{formatWalletMoney(row.gift_balance, row.currency)}</TableCell>
      <TableCell>
        <Label color={row.status === 'active' ? 'success' : 'default'} variant="soft">
          {walletStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatWalletDateTime(row.updated_at, locale)}
      </TableCell>
      <TableCell align="right">
        <Stack direction="row" justifyContent="flex-end" spacing={1}>
          <Button size="small" variant="outlined" onClick={() => onOpenLedger(row)}>
            {t('adminWallets.actions.ledger')}
          </Button>
          <Button size="small" variant="contained" startIcon={<Iconify icon="solar:pen-bold" />} onClick={() => onOpenAdjust(row)}>
            {t('adminWallets.actions.adjust')}
          </Button>
        </Stack>
      </TableCell>
    </TableRow>
  );
}

function OwnerCell({ t, row }: { t: TFunction<'admin'>; row: AdminWallet }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="body2" sx={{ fontWeight: 600 }}>
        {adminWalletOwner(row) || t('wallet.emptyValue')}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {row.owner_email}
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
    { id: 'owner', label: t('adminWallets.fields.owner'), width: 240 },
    { id: 'balance', label: t('wallet.metrics.availableBalance'), width: 150 },
    { id: 'recharge', label: t('wallet.metrics.rechargeBalance'), width: 150 },
    { id: 'gift', label: t('wallet.metrics.giftBalance'), width: 150 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'updated_at', label: t('adminWallets.fields.updatedAt'), width: 180 },
    { id: 'action', label: t('wallet.table.action'), width: 190, align: 'right' },
  ];
}
