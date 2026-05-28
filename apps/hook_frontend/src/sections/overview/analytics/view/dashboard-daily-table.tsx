import type { TFunction } from 'i18next';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';
import type { DashboardDailyStat, DashboardDailyStats } from 'src/types/dashboard';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';

import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableSkeleton,
  TableHeadCustom,
  TablePaginationCustom,
} from 'src/components/table';

import {
  formatMs,
  formatInteger,
  formatDashboardCost,
  formatDashboardTokens,
} from './dashboard-format';

type DailyTableProps = {
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
  loading: boolean;
  table: UseTableReturn;
  data?: DashboardDailyStats;
};

const DAILY_TABLE_MIN_WIDTH = 960;
const MODEL_NAME_MAX_WIDTH = 220;
const DAILY_TABLE_ROWS_PER_PAGE_OPTIONS = [10, 25, 50];

export function DailyStatsTable({ t, locale, isAdmin, loading, table, data }: DailyTableProps) {
  const rows = data?.day_page.items ?? [];
  const total = data?.day_page.total ?? 0;
  const head = tableHead(t, isAdmin);

  return (
    <Card>
      <CardHeader title={t('dashboard.stats.daily.tableTitle')} sx={{ pb: 2 }} />
      <Scrollbar sx={{ pt: 1 }}>
        <Table sx={{ minWidth: DAILY_TABLE_MIN_WIDTH }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {loading ? <TableSkeleton rowCount={table.rowsPerPage} cellCount={head.length} /> : null}
            {!loading ? rows.map((row) => <DailyRow key={row.date} row={row} t={t} locale={locale} isAdmin={isAdmin} />) : null}
            <TableNoData title={t('dashboard.stats.empty')} notFound={!loading && total === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        rowsPerPageOptions={DAILY_TABLE_ROWS_PER_PAGE_OPTIONS}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </Card>
  );
}

function DailyRow({
  row,
  t,
  locale,
  isAdmin,
}: {
  row: DashboardDailyStat;
  t: TFunction<'admin'>;
  locale: string;
  isAdmin: boolean;
}) {
  return (
    <TableRow hover>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatDate(row.date, locale)}</TableCell>
      <TableCell align="right">{formatInteger(row.request_count, locale)}</TableCell>
      <TableCell align="right">{formatDashboardTokens(row.total_tokens)}</TableCell>
      <TableCell align="right">{formatDashboardCost(row.total_cost)}</TableCell>
      <TableCell align="right">{formatMs(row.avg_latency_ms)}</TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: MODEL_NAME_MAX_WIDTH }}>
          {modelText(row, t)}
        </Typography>
      </TableCell>
      {isAdmin ? <TableCell align="right">{formatInteger(row.unique_providers, locale)}</TableCell> : null}
    </TableRow>
  );
}

function tableHead(t: TFunction<'admin'>, isAdmin: boolean): TableHeadCellProps[] {
  const head = [
    { id: 'date', label: t('dashboard.stats.daily.date') },
    { id: 'requests', label: t('dashboard.stats.columns.requests'), align: 'right' as const },
    { id: 'tokens', label: t('dashboard.stats.columns.tokens'), align: 'right' as const },
    { id: 'cost', label: t('dashboard.stats.columns.cost'), align: 'right' as const },
    { id: 'avgLatency', label: t('dashboard.stats.columns.avgLatency'), align: 'right' as const },
    { id: 'models', label: t('dashboard.stats.daily.models') },
  ];
  if (isAdmin) {
    head.push({
      id: 'providers',
      label: t('dashboard.stats.daily.providers'),
      align: 'right' as const,
    });
  }
  return head;
}

function modelText(row: DashboardDailyStat, t: TFunction<'admin'>) {
  const first = row.model_breakdown[0]?.name;
  if (!first) return t('dashboard.stats.empty');
  if (row.unique_models <= 1) return first;
  return t('dashboard.stats.daily.modelCount', { first, count: row.unique_models });
}

function formatDate(date: string, locale: string) {
  return new Intl.DateTimeFormat(locale, {
    month: '2-digit',
    day: '2-digit',
    year: 'numeric',
  }).format(new Date(`${date}T00:00:00`));
}
