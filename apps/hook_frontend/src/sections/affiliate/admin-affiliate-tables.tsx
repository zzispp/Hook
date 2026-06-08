'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type {
  AdminAffiliateRelation,
  AdminAffiliateCommission,
  AdminAffiliateDailyReportItem,
  AdminAffiliateReferrerReportItem,
} from 'src/types/affiliate';

import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableHeadCustom,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { formatDate, formatMoney, formatCount, formatPercent } from './admin-affiliate-format';

type PageProps = {
  t: TFunction<'admin'>;
  locale: string;
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RelationsTable(props: PageProps & {
  rows: AdminAffiliateRelation[];
  onRebind: (row: AdminAffiliateRelation) => void;
  onClear: (row: AdminAffiliateRelation) => void;
}) {
  const head = relationHead(props.t);
  return (
    <PagedTable {...props} head={head} emptyText={props.t('adminAffiliates.empty.relations')} minWidth={1380}>
      {props.rows.map((row) => (
        <TableRow hover key={row.user.id}>
          <UserCell user={row.user} />
          <UserCell user={row.referrer} />
          <NoWrapCell>{formatDate(row.referred_at, props.locale)}</NoWrapCell>
          <TableCell align="right">{formatMoney(row.referred_recharge_amount, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.commission_amount, props.locale)}</TableCell>
          <NoWrapCell>{formatDate(row.last_commission_at, props.locale)}</NoWrapCell>
          <TableCell align="left" sx={tableStickyActionCellSx}>
            <Stack direction="row" spacing={1} justifyContent="flex-end" sx={{ flexWrap: 'nowrap' }}>
              <Button
                size="small"
                startIcon={<Iconify icon="solar:pen-bold" />}
                sx={{ flexShrink: 0, whiteSpace: 'nowrap' }}
                onClick={() => props.onRebind(row)}
              >
                {props.t('adminAffiliates.actions.rebind')}
              </Button>
              <Button
                size="small"
                color="warning"
                startIcon={<Iconify icon="solar:eraser-bold" />}
                sx={{ flexShrink: 0, whiteSpace: 'nowrap' }}
                onClick={() => props.onClear(row)}
              >
                {props.t('adminAffiliates.actions.clear')}
              </Button>
            </Stack>
          </TableCell>
        </TableRow>
      ))}
    </PagedTable>
  );
}

export function CommissionsTable(props: PageProps & { rows: AdminAffiliateCommission[] }) {
  const head = commissionHead(props.t);
  return (
    <PagedTable {...props} head={head} emptyText={props.t('adminAffiliates.empty.commissions')} minWidth={2510}>
      {props.rows.map((row) => (
        <TableRow hover key={row.id}>
          <NoWrapCell>{row.id}</NoWrapCell>
          <UserCell user={row.referrer} />
          <UserCell user={row.referred} />
          <NoWrapCell>{row.recharge_order_no || row.recharge_order_id}</NoWrapCell>
          <TableCell align="right">{formatMoney(row.payable_amount, props.locale)}</TableCell>
          <TableCell align="right">{formatPercent(row.commission_percent, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.commission_amount, props.locale)}</TableCell>
          <TableCell>
            <StatusChip t={props.t} status={row.status} />
          </TableCell>
          <NoWrapCell>{failureReasonLabel(props.t, row.failure_reason)}</NoWrapCell>
          <NoWrapCell>{row.wallet_transaction_id || '-'}</NoWrapCell>
          <NoWrapCell>{formatDate(row.created_at, props.locale)}</NoWrapCell>
        </TableRow>
      ))}
    </PagedTable>
  );
}

export function DailyReportTable(props: Pick<PageProps, 't' | 'locale' | 'loading' | 'rowsPerPage'> & { rows: AdminAffiliateDailyReportItem[] }) {
  const head = dailyHead(props.t);
  return (
    <SimpleTable t={props.t} loading={props.loading} rowsPerPage={props.rowsPerPage} head={head} emptyText={props.t('adminAffiliates.empty.daily')} minWidth={760}>
      {props.rows.map((row) => (
        <TableRow hover key={row.date}>
          <TableCell>{row.date}</TableCell>
          <TableCell align="right">{formatCount(row.commission_order_count, props.locale)}</TableCell>
          <TableCell align="right">{formatCount(row.referred_payer_count, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.payable_amount, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.commission_amount, props.locale)}</TableCell>
        </TableRow>
      ))}
    </SimpleTable>
  );
}

export function ReferrerReportTable(props: PageProps & { rows: AdminAffiliateReferrerReportItem[] }) {
  const head = referrerHead(props.t);
  return (
    <PagedTable {...props} head={head} emptyText={props.t('adminAffiliates.empty.referrers')} minWidth={920}>
      {props.rows.map((row) => (
        <TableRow hover key={row.referrer.id}>
          <UserCell user={row.referrer} />
          <TableCell align="right">{formatCount(row.referred_user_count, props.locale)}</TableCell>
          <TableCell align="right">{formatCount(row.commission_order_count, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.payable_amount, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.commission_amount, props.locale)}</TableCell>
        </TableRow>
      ))}
    </PagedTable>
  );
}

function PagedTable(props: PageProps & {
  head: TableHeadCellProps[];
  emptyText: string;
  minWidth: number;
  children: React.ReactNode;
}) {
  return (
    <>
      <SimpleTable {...props}>{props.children}</SimpleTable>
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

function SimpleTable({
  t,
  head,
  loading,
  rowsPerPage,
  emptyText,
  minWidth,
  children,
}: {
  t: TFunction<'admin'>;
  head: TableHeadCellProps[];
  loading: boolean;
  rowsPerPage: number;
  emptyText: string;
  minWidth: number;
  children: React.ReactNode;
}) {
  return (
    <Scrollbar>
      <Table
        sx={{
          width: minWidth,
          minWidth,
          tableLayout: 'fixed',
          '& .MuiTableCell-root': { verticalAlign: 'top' },
        }}
      >
        <TableHeadCustom
          headCells={head}
          sx={{
            '& .MuiTableCell-root': {
              whiteSpace: 'nowrap',
            },
          }}
        />
        <TableBody>
          {loading ? <LoadingRows t={t} head={head} rows={rowsPerPage} /> : children}
          <TableNoData title={emptyText} notFound={!loading && !hasChildren(children)} />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function UserCell({ user }: { user: { username: string; email: string; id: string; affiliate_code: string } | null }) {
  if (!user) return <TableCell>-</TableCell>;
  return (
    <TableCell>
      <Stack spacing={0.5}>
        <Typography variant="body2" sx={{ fontWeight: 600 }}>
          {user.username || user.id}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {user.email}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {user.affiliate_code}
        </Typography>
      </Stack>
    </TableCell>
  );
}

function NoWrapCell({ children }: { children: React.ReactNode }) {
  return <TableCell sx={{ whiteSpace: 'nowrap' }}>{children}</TableCell>;
}

function StatusChip({ t, status }: { t: TFunction<'admin'>; status: string }) {
  const color = status === 'success' ? 'success' : 'error';
  return (
    <Chip
      size="small"
      color={color}
      variant="soft"
      label={statusLabel(t, status)}
      sx={{ whiteSpace: 'nowrap' }}
    />
  );
}

function statusLabel(t: TFunction<'admin'>, status: string) {
  if (status === 'success') return t('adminAffiliates.status.success');
  if (status === 'failed') return t('adminAffiliates.status.failed');
  return status;
}

function failureReasonLabel(t: TFunction<'admin'>, reason: string | null) {
  if (!reason) return '-';
  if (reason === 'below_min_commission_amount') {
    return t('adminAffiliates.failureReasons.belowMinCommissionAmount');
  }
  return reason;
}

function LoadingRows({ t, rows, head }: { t: TFunction<'admin'>; rows: number; head: TableHeadCellProps[] }) {
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

function hasChildren(children: React.ReactNode) {
  return Array.isArray(children) ? children.length > 0 : Boolean(children);
}

function relationHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'user', label: t('adminAffiliates.fields.user'), width: 230 },
    { id: 'referrer', label: t('adminAffiliates.fields.referrer'), width: 230 },
    { id: 'referred_at', label: t('adminAffiliates.fields.referredAt'), width: 180 },
    { id: 'referred_recharge_amount', label: t('adminAffiliates.fields.referredRechargeAmount'), width: 160, align: 'right' },
    { id: 'commission_amount', label: t('adminAffiliates.fields.commissionAmount'), width: 150, align: 'right' },
    { id: 'last_commission_at', label: t('adminAffiliates.fields.lastCommissionAt'), width: 180 },
    withStickyActionHeadCell({
      id: 'actions',
      label: t('common.actions'),
      width: 260,
      align: 'left',
    }),
  ];
}

function commissionHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'id', label: t('adminAffiliates.fields.commissionId'), width: 330 },
    { id: 'referrer', label: t('adminAffiliates.fields.referrer'), width: 260 },
    { id: 'referred', label: t('adminAffiliates.fields.referred'), width: 260 },
    { id: 'order', label: t('adminAffiliates.fields.rechargeOrderId'), width: 300 },
    { id: 'payable', label: t('adminAffiliates.fields.payableAmount'), width: 160, align: 'right' },
    { id: 'percent', label: t('adminAffiliates.fields.commissionPercent'), width: 150, align: 'right' },
    { id: 'amount', label: t('adminAffiliates.fields.commissionAmount'), width: 160, align: 'right' },
    { id: 'status', label: t('adminAffiliates.fields.status'), width: 130 },
    { id: 'failure_reason', label: t('adminAffiliates.fields.failureReason'), width: 260 },
    { id: 'wallet_transaction', label: t('adminAffiliates.fields.walletTransactionId'), width: 330 },
    { id: 'created_at', label: t('adminAffiliates.fields.createdAt'), width: 250 },
  ];
}

function dailyHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'date', label: t('adminAffiliates.fields.date'), width: 150 },
    { id: 'orders', label: t('adminAffiliates.fields.commissionOrderCount'), width: 160, align: 'right' },
    { id: 'payers', label: t('adminAffiliates.fields.referredPayerCount'), width: 180, align: 'right' },
    { id: 'payable', label: t('adminAffiliates.fields.payableAmount'), width: 150, align: 'right' },
    { id: 'commission', label: t('adminAffiliates.fields.commissionAmount'), width: 150, align: 'right' },
  ];
}

function referrerHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'referrer', label: t('adminAffiliates.fields.referrer'), width: 260 },
    { id: 'referred_users', label: t('adminAffiliates.fields.referredUserCount'), width: 140, align: 'right' },
    { id: 'orders', label: t('adminAffiliates.fields.commissionOrderCount'), width: 150, align: 'right' },
    { id: 'payable', label: t('adminAffiliates.fields.payableAmount'), width: 150, align: 'right' },
    { id: 'commission', label: t('adminAffiliates.fields.commissionAmount'), width: 150, align: 'right' },
  ];
}
