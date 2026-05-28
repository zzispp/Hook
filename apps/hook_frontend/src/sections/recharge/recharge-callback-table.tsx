'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { PaymentCallbackRecord } from 'src/types/recharge';

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
  formatRechargeDate,
  callbackStatusColor,
  paymentCallbackStatusLabel,
} from './recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rows: PaymentCallbackRecord[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onOpen: (record: PaymentCallbackRecord) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RechargeCallbackTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {props.t('adminRecharges.summary.callbacks', {
          shown: props.rows.length,
          total: props.total,
        })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1180 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading
              ? props.rows.map((row) => <CallbackRow key={row.id} row={row} {...props} />)
              : null}
            <TableNoData
              title={props.t('adminRecharges.empty.callbacks')}
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

function CallbackRow({
  t,
  row,
  locale,
  onOpen,
}: Pick<Props, 't' | 'locale' | 'onOpen'> & { row: PaymentCallbackRecord }) {
  return (
    <TableRow hover sx={{ cursor: 'pointer' }} onClick={() => onOpen(row)}>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {row.order_no || '-'}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {formatRechargeDate(row.received_at, locale)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{row.payment_channel_code}</TableCell>
      <TableCell>{t(`adminRecharges.callbackKind.${row.callback_kind}`)}</TableCell>
      <TableCell>{row.http_method}</TableCell>
      <TableCell>
        <Label color={callbackStatusColor(row.status)} variant="soft">
          {paymentCallbackStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell>
        <Label color={row.settled ? 'success' : 'default'} variant="soft">
          {row.settled
            ? t('adminRecharges.settlement.settled')
            : t('adminRecharges.settlement.unsettled')}
        </Label>
      </TableCell>
      <TableCell>{row.provider_trade_no || '-'}</TableCell>
      <TableCell>{row.payment_method || '-'}</TableCell>
      <TableCell>{row.trade_status || '-'}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {row.processed_at ? formatRechargeDate(row.processed_at, locale) : '-'}
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

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'order_no', label: t('adminRecharges.fields.orderNo'), width: 210 },
    { id: 'payment_channel', label: t('adminRecharges.fields.paymentChannel'), width: 130 },
    { id: 'callback_kind', label: t('adminRecharges.fields.callbackKind'), width: 110 },
    { id: 'http_method', label: t('adminRecharges.fields.httpMethod'), width: 100 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'settled', label: t('adminRecharges.fields.settled'), width: 110 },
    { id: 'provider_trade_no', label: t('adminRecharges.fields.providerTradeNo'), width: 180 },
    { id: 'payment_method', label: t('adminRecharges.fields.paymentMethod'), width: 120 },
    { id: 'trade_status', label: t('adminRecharges.fields.tradeStatus'), width: 120 },
    { id: 'processed_at', label: t('adminRecharges.fields.processedAt'), width: 180 },
  ];
}
