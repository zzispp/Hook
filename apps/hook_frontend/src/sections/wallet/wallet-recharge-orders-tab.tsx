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
} from 'src/sections/recharge/recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  orders: RechargeOrder[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function WalletRechargeOrdersTab({
  t,
  page,
  total,
  locale,
  orders,
  loading,
  rowsPerPage,
  onPageChange,
  onRowsPerPageChange,
}: Props) {
  const head = tableHead(t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {t('adminRecharges.summary.orders', {
          shown: orders.length,
          total,
        })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1080 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {loading ? <LoadingRows t={t} head={head} rows={rowsPerPage} /> : null}
            {!loading
              ? orders.map((order) => (
                  <OrderRow key={order.id} t={t} locale={locale} order={order} />
                ))
              : null}
            <TableNoData
              title={t('wallet.recharge.emptyOrders')}
              notFound={!loading && orders.length === 0}
            />
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

function OrderRow({
  t,
  locale,
  order,
}: {
  t: TFunction<'admin'>;
  locale: string;
  order: RechargeOrder;
}) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {order.order_no}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {formatRechargeDate(order.created_at, locale)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{order.package_name}</TableCell>
      <TableCell>{formatUsd(order.recharge_amount)}</TableCell>
      <TableCell>{formatUsd(order.gift_amount)}</TableCell>
      <TableCell>{formatUsd(order.total_arrival_amount)}</TableCell>
      <TableCell>{formatCny(order.payable_amount)}</TableCell>
      <TableCell>
        <Label color={orderStatusColor(order.status)} variant="soft">
          {rechargeOrderStatusLabel(t, order.status)}
        </Label>
      </TableCell>
      <TableCell>{paymentMethodText(order)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatRechargeDate(order.expires_at, locale)}
      </TableCell>
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

function paymentMethodText(order: RechargeOrder) {
  return order.payment_method || order.payment_channel_name || order.payment_channel_code || '-';
}

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'order_no', label: t('adminRecharges.fields.orderNo'), width: 210 },
    { id: 'package_name', label: t('adminRecharges.fields.packageName'), width: 180 },
    { id: 'recharge_amount', label: t('adminRecharges.fields.rechargeAmount'), width: 120 },
    { id: 'gift_amount', label: t('adminRecharges.fields.giftAmount'), width: 120 },
    { id: 'total_arrival_amount', label: t('adminRecharges.fields.totalArrival'), width: 130 },
    { id: 'payable_amount', label: t('adminRecharges.fields.payableAmount'), width: 120 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'payment_method', label: t('wallet.recharge.paymentMethod'), width: 130 },
    { id: 'expires_at', label: t('adminRecharges.fields.expiresAt'), width: 180 },
  ];
}
