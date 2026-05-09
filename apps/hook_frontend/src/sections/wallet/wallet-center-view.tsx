'use client';

import type { IconifyProps } from 'src/components/iconify';
import type { TableHeadCellProps } from 'src/components/table';
import type { WalletSummary, WalletTransaction } from 'src/types/wallet';

import { useMemo, useCallback } from 'react';

import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useWalletBalance, useWalletTransactions } from 'src/actions/wallet';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';
import { useTable, TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

const TABLE_HEAD: TableHeadCellProps[] = [
  { id: 'created_at', label: '时间', width: 180 },
  { id: 'category', label: '类型', width: 160 },
  { id: 'amount', label: '变动', width: 140 },
  { id: 'balance', label: '余额变化', width: 220 },
  { id: 'description', label: '说明' },
];

type BalanceCardItem = {
  label: string;
  value: string;
  icon: IconifyProps['icon'];
};

type TransactionTableProps = {
  loading: boolean;
  items: WalletTransaction[];
  total: number;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function WalletCenterView() {
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'created_at' });
  const balance = useWalletBalance();
  const transactions = useWalletTransactions(table.page, table.rowsPerPage);
  const wallet = balance.wallet ?? transactions.wallet;
  const loading = balance.isLoading || transactions.isLoading;
  const errorMessage = balance.error?.message || transactions.error?.message;
  const handleRefresh = useCallback(() => {
    balance.refresh();
    transactions.refresh();
  }, [balance, transactions]);

  const stats = useMemo(() => walletStats(wallet), [wallet]);

  return (
    <DashboardContent maxWidth="xl">
      <WalletBreadcrumbs loading={balance.isValidating || transactions.isValidating} onRefresh={handleRefresh} />

      {errorMessage ? (
        <Alert severity="error" sx={{ mb: 3 }}>
          {errorMessage}
        </Alert>
      ) : null}

      <BalanceCards items={stats} />

      <TransactionTable
        loading={loading}
        items={transactions.items}
        total={transactions.total}
        page={table.page}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </DashboardContent>
  );
}

function WalletBreadcrumbs({ loading, onRefresh }: { loading: boolean; onRefresh: VoidFunction }) {
  const { t } = useTranslate('admin');

  return (
    <CustomBreadcrumbs
      heading={t('nav.walletCenter')}
      links={[
        { name: t('nav.dashboard'), href: paths.dashboard.root },
        { name: t('nav.account') },
        { name: t('nav.walletCenter') },
      ]}
      action={
        <Button variant="outlined" loading={loading} startIcon={<Iconify icon="solar:restart-bold" />} onClick={onRefresh}>
          刷新
        </Button>
      }
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

function walletStats(wallet?: WalletSummary): BalanceCardItem[] {
  return [
    { label: '可用余额', value: formatCurrency(wallet?.balance), icon: 'solar:wad-of-money-bold' },
    { label: '充值余额', value: formatCurrency(wallet?.recharge_balance), icon: 'solar:bill-list-bold' },
    { label: '赠款余额', value: formatCurrency(wallet?.gift_balance), icon: 'solar:cup-star-bold' },
    { label: '累计消费', value: formatCurrency(wallet?.total_consumed), icon: 'solar:cart-3-bold' },
  ];
}

function BalanceCards({ items }: { items: BalanceCardItem[] }) {
  return (
    <Grid container spacing={3} sx={{ mb: 3 }}>
      {items.map((item) => (
        <Grid key={item.label} size={{ xs: 12, sm: 6, md: 3 }}>
          <BalanceCard {...item} />
        </Grid>
      ))}
    </Grid>
  );
}

function BalanceCard({ label, value, icon }: BalanceCardItem) {
  return (
    <Card sx={{ p: 2.5 }}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
        <Stack spacing={0.5}>
          <Typography variant="overline" sx={{ color: 'text.secondary' }}>
            {label}
          </Typography>
          <Typography variant="h5">{value}</Typography>
        </Stack>
        <Iconify icon={icon} width={28} sx={{ color: 'primary.main' }} />
      </Stack>
    </Card>
  );
}

function TransactionRow({ transaction }: { transaction: WalletTransaction }) {
  const positive = transaction.amount >= 0;

  return (
    <TableRow hover>
      <TableCell sx={{ color: 'text.secondary', whiteSpace: 'nowrap' }}>
        {formatDateTime(transaction.created_at)}
      </TableCell>
      <TableCell>
        <Stack spacing={0.5} alignItems="flex-start">
          <Label color={transactionColor(transaction.category)} variant="soft">
            {transactionCategory(transaction.category)}
          </Label>
          <Typography variant="caption" color="text.secondary">
            {transactionReason(transaction.reason_code)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell sx={{ color: positive ? 'success.main' : 'error.main', fontWeight: 700 }}>
        {positive ? '+' : ''}
        {formatCurrency(transaction.amount)}
      </TableCell>
      <TableCell sx={{ fontFamily: 'monospace', whiteSpace: 'nowrap' }}>
        {formatCurrency(transaction.balance_before)} {'->'} {formatCurrency(transaction.balance_after)}
      </TableCell>
      <TableCell sx={{ color: 'text.secondary' }}>{transaction.description || '-'}</TableCell>
    </TableRow>
  );
}

function TransactionTable({ loading, items, total, page, rowsPerPage, onPageChange, onRowsPerPageChange }: TransactionTableProps) {
  return (
    <Card>
      <TransactionTableHeader total={total} />

      <Scrollbar>
        <Table sx={{ minWidth: 920 }}>
          <TableHeadCustom headCells={TABLE_HEAD} />
          <TableBody>
            {loading ? <LoadingRows rows={rowsPerPage} /> : null}
            {!loading ? items.map((transaction) => <TransactionRow key={transaction.id} transaction={transaction} />) : null}
            <TableNoData title="暂无资金流水" notFound={!loading && items.length === 0} />
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
    </Card>
  );
}

function TransactionTableHeader({ total }: { total: number }) {
  return (
    <Stack spacing={0.5} sx={{ p: 2.5 }}>
      <Typography variant="h6">资金流水</Typography>
      <Typography variant="body2" color="text.secondary">
        共 {total} 条
      </Typography>
    </Stack>
  );
}

function LoadingRows({ rows }: { rows: number }) {
  return (
    <>
      {Array.from({ length: rows }).map((_, index) => (
        <TableRow key={index}>
          {TABLE_HEAD.map((cell) => (
            <TableCell key={cell.id || cell.label?.toString()} sx={{ color: 'text.disabled' }}>
              加载中...
            </TableCell>
          ))}
        </TableRow>
      ))}
    </>
  );
}

function transactionCategory(category: string) {
  const labels: Record<string, string> = {
    recharge: '充值',
    gift: '赠款',
    adjust: '调账',
    refund: '退款',
    consume: '消费',
  };
  return labels[category] ?? category;
}

function transactionReason(reason: string) {
  const labels: Record<string, string> = {
    adjust_admin: '管理员调账',
    wallet_created: '钱包初始化',
  };
  return labels[reason] ?? reason;
}

function transactionColor(category: string) {
  if (category === 'refund' || category === 'consume') return 'error';
  if (category === 'recharge') return 'success';
  if (category === 'gift') return 'info';
  return 'warning';
}

function formatCurrency(value?: number | null) {
  return new Intl.NumberFormat('zh-CN', {
    style: 'currency',
    currency: 'CNY',
    minimumFractionDigits: 2,
  }).format(value ?? 0);
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat('zh-CN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}
