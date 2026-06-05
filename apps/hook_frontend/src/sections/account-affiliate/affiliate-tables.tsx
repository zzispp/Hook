'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { AffiliateReferral, AffiliateCommission } from 'src/types/account-affiliate';

import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { formatDate, formatMoney, formatPercent } from './affiliate-format';

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

export function ReferralsTable(props: PageProps & { rows: AffiliateReferral[] }) {
  return (
    <PagedTable
      {...props}
      head={referralHead(props.t)}
      emptyText={props.t('affiliateCenter.empty.referrals')}
      minWidth={920}
    >
      {props.rows.map((row) => (
        <TableRow hover key={row.referred_user_id}>
          <UserCell username={row.username} email={row.masked_email} id={row.referred_user_id} />
          <NoWrapCell>{formatDate(row.referred_at, props.locale)}</NoWrapCell>
          <TableCell align="right">{formatMoney(row.referred_recharge_amount, props.locale)}</TableCell>
          <TableCell align="right">{formatMoney(row.commission_amount, props.locale)}</TableCell>
          <NoWrapCell>{formatDate(row.last_commission_at, props.locale)}</NoWrapCell>
        </TableRow>
      ))}
    </PagedTable>
  );
}

export function CommissionsTable(props: PageProps & { rows: AffiliateCommission[] }) {
  return (
    <PagedTable
      {...props}
      head={commissionHead(props.t)}
      emptyText={props.t('affiliateCenter.empty.commissions')}
      minWidth={2050}
    >
      {props.rows.map((row) => (
        <TableRow hover key={row.id}>
          <NoWrapCell>{row.id}</NoWrapCell>
          <UserCell
            username={row.referred.username}
            email={row.referred.masked_email}
            id={row.referred.referred_user_id}
          />
          <NoWrapCell>{row.recharge_order_no}</NoWrapCell>
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

function PagedTable(props: PageProps & {
  head: TableHeadCellProps[];
  emptyText: string;
  minWidth: number;
  children: React.ReactNode;
}) {
  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: props.minWidth, '& .MuiTableCell-root': { verticalAlign: 'top' } }}>
          <TableHeadCustom headCells={props.head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={props.head} rows={props.rowsPerPage} /> : props.children}
            <TableNoData title={props.emptyText} notFound={!props.loading && !hasChildren(props.children)} />
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

function UserCell({ username, email, id }: { username: string; email: string; id: string }) {
  return (
    <TableCell>
      <Stack spacing={0.5}>
        <Typography variant="body2" sx={{ fontWeight: 600 }}>
          {username || id}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {email}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {id}
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
  if (status === 'success') return t('affiliateCenter.status.success');
  if (status === 'failed') return t('affiliateCenter.status.failed');
  return status;
}

function failureReasonLabel(t: TFunction<'admin'>, reason: string | null) {
  if (!reason) return '-';
  if (reason === 'below_min_commission_amount') {
    return t('affiliateCenter.failureReasons.belowMinCommissionAmount');
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

function referralHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'user', label: t('affiliateCenter.fields.referredUser'), width: 260 },
    { id: 'referred_at', label: t('affiliateCenter.fields.referredAt'), width: 180 },
    { id: 'recharge', label: t('affiliateCenter.fields.referredRechargeAmount'), width: 170, align: 'right' },
    { id: 'commission', label: t('affiliateCenter.fields.commissionAmount'), width: 150, align: 'right' },
    { id: 'last_commission_at', label: t('affiliateCenter.fields.lastCommissionAt'), width: 180 },
  ];
}

function commissionHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'id', label: t('affiliateCenter.fields.commissionId'), width: 330 },
    { id: 'referred', label: t('affiliateCenter.fields.referredUser'), width: 260 },
    { id: 'order', label: t('affiliateCenter.fields.rechargeOrderNo'), width: 260 },
    { id: 'payable', label: t('affiliateCenter.fields.payableAmount'), width: 150, align: 'right' },
    { id: 'percent', label: t('affiliateCenter.fields.commissionPercent'), width: 150, align: 'right' },
    { id: 'amount', label: t('affiliateCenter.fields.commissionAmount'), width: 160, align: 'right' },
    { id: 'status', label: t('affiliateCenter.fields.status'), width: 130 },
    { id: 'failure_reason', label: t('affiliateCenter.fields.failureReason'), width: 260 },
    { id: 'wallet_transaction', label: t('affiliateCenter.fields.walletTransactionId'), width: 330 },
    { id: 'created_at', label: t('affiliateCenter.fields.createdAt'), width: 250 },
  ];
}
