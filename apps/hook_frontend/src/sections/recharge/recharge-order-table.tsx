'use client';

import type { TFunction } from 'i18next';
import type { RechargeOrder } from 'src/types/recharge';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import {
  formatCny,
  formatUsd,
  orderStatusColor,
  formatRechargeDate,
  rechargeOrderStatusLabel,
} from './recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rows: RechargeOrder[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RechargeOrderTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {props.t('adminRecharges.summary.orders', {
          shown: props.rows.length,
          total: props.total,
        })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1240 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading
              ? props.rows.map((row) => <OrderRow key={row.id} row={row} {...props} />)
              : null}
            <TableNoData
              title={props.t('adminRecharges.empty.orders')}
              notFound={!props.loading && props.rows.length === 0}
            />
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

function OrderRow({
  t,
  row,
  locale,
}: Pick<Props, 't' | 'locale'> & { row: RechargeOrder }) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {row.order_no}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {formatRechargeDate(row.created_at, locale)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2">{row.username || row.user_id}</Typography>
          <Typography variant="caption" color="text.secondary">
            {row.user_email || '-'}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{row.package_name}</TableCell>
      <TableCell>{formatUsd(row.recharge_amount)}</TableCell>
      <TableCell>{formatUsd(row.gift_amount)}</TableCell>
      <TableCell>{formatUsd(row.total_arrival_amount)}</TableCell>
      <TableCell>{formatCny(row.payable_amount)}</TableCell>
      <TableCell>
        <Label color={orderStatusColor(row.status)} variant="soft">
          {rechargeOrderStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell>{row.payment_channel_name || row.payment_channel_code || '-'}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatRechargeDate(row.expires_at, locale)}</TableCell>
    </TableRow>
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
    { id: 'order_no', label: t('adminRecharges.fields.orderNo'), width: 210 },
    { id: 'user', label: t('common.user'), width: 210 },
    { id: 'package_name', label: t('adminRecharges.fields.packageName'), width: 180 },
    { id: 'recharge_amount', label: t('adminRecharges.fields.rechargeAmount'), width: 120 },
    { id: 'gift_amount', label: t('adminRecharges.fields.giftAmount'), width: 120 },
    { id: 'total_arrival_amount', label: t('adminRecharges.fields.totalArrival'), width: 130 },
    { id: 'payable_amount', label: t('adminRecharges.fields.payableAmount'), width: 120 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'payment_channel', label: t('adminRecharges.fields.paymentChannel'), width: 150 },
    { id: 'expires_at', label: t('adminRecharges.fields.expiresAt'), width: 180 },
  ];
}
