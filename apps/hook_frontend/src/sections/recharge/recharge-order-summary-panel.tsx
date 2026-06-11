'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { RechargeOrderSummaryResponse } from 'src/types/recharge';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { formatCny, formatRechargeDate } from './recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  data?: RechargeOrderSummaryResponse;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RechargeOrderSummaryPanel(props: Props) {
  const head = tableHead(props.t);
  const rows = props.data?.items ?? [];
  const total = props.data?.total ?? 0;

  return (
    <Box sx={{ px: 2.5, pb: 2.5 }}>
      <SummaryOverview t={props.t} locale={props.locale} data={props.data} loading={props.loading} />
      <Box sx={{ mt: 2, border: (theme) => `solid 1px ${theme.vars.palette.divider}`, borderRadius: 1 }}>
        <Stack spacing={0.5} sx={{ p: 2 }}>
          <Typography variant="subtitle2">{props.t('adminRecharges.summary.userSummaryTitle')}</Typography>
          <Typography variant="body2" color="text.secondary">
            {props.t('adminRecharges.summary.userSummaryRows', { shown: rows.length, total })}
          </Typography>
        </Stack>
        <Scrollbar>
          <Table sx={{ minWidth: 720 }}>
            <TableHeadCustom headCells={head} />
            <TableBody>
              {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
              {!props.loading
                ? rows.map((row) => (
                    <TableRow key={row.user_id} hover>
                      <TableCell>
                        <Stack spacing={0.5}>
                          <Typography variant="body2" sx={{ fontWeight: 600 }}>
                            {row.username || row.user_id}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {row.user_email || '-'}
                          </Typography>
                        </Stack>
                      </TableCell>
                      <TableCell>{formatInteger(row.order_count, props.locale)}</TableCell>
                      <TableCell>{formatCny(row.total_payable_amount)}</TableCell>
                      <TableCell sx={{ whiteSpace: 'nowrap' }}>
                        {row.last_paid_at ? formatRechargeDate(row.last_paid_at, props.locale) : '-'}
                      </TableCell>
                    </TableRow>
                  ))
                : null}
              <TableNoData
                title={props.t('adminRecharges.empty.userSummary')}
                notFound={!props.loading && rows.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
        <TablePaginationCustom
          page={props.page}
          count={total}
          rowsPerPage={props.rowsPerPage}
          onPageChange={props.onPageChange}
          onRowsPerPageChange={props.onRowsPerPageChange}
        />
      </Box>
    </Box>
  );
}

function SummaryOverview({
  t,
  locale,
  data,
  loading,
}: {
  t: TFunction<'admin'>;
  locale: string;
  data?: RechargeOrderSummaryResponse;
  loading: boolean;
}) {
  const cards = [
    {
      label: t('adminRecharges.summary.totalPayableAmount'),
      value: formatCny(data?.summary.total_payable_amount),
    },
    {
      label: t('adminRecharges.summary.orderCount'),
      value: formatInteger(data?.summary.order_count, locale),
    },
    {
      label: t('adminRecharges.summary.userCount'),
      value: formatInteger(data?.summary.user_count, locale),
    },
  ];

  return (
    <Box
      sx={{
        gap: 2,
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', md: 'repeat(3, minmax(0, 1fr))' },
      }}
    >
      {cards.map((card) => (
        <Box
          key={card.label}
          sx={{
            p: 2,
            borderRadius: 1,
            bgcolor: 'background.neutral',
            border: (theme) => `solid 1px ${theme.vars.palette.divider}`,
          }}
        >
          <Typography variant="caption" color="text.secondary">
            {card.label}
          </Typography>
          <Typography variant="h6">{loading ? t('common.loading') : card.value}</Typography>
        </Box>
      ))}
    </Box>
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
    { id: 'user', label: t('common.user'), width: 240 },
    { id: 'order_count', label: t('adminRecharges.summary.orderCount'), width: 140 },
    { id: 'total_payable_amount', label: t('adminRecharges.summary.totalPayableAmount'), width: 180 },
    { id: 'last_paid_at', label: t('adminRecharges.summary.lastPaidAt'), width: 180 },
  ];
}

function formatInteger(value: number | undefined, locale: string) {
  return new Intl.NumberFormat(locale).format(value ?? 0);
}
