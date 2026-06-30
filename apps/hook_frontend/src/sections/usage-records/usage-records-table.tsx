'use client';

import type { UsageRecord } from 'src/types/provider';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { formatApiFormat } from 'src/sections/admin/provider-management-utils';
import { TableLoadingRows, ManagementTableHead } from 'src/sections/admin/shared';
import {
  RequestRecordDurationText,
  useRequestRecordDurationNow,
} from 'src/sections/admin/request-record-duration-text';
import {
  formatCost,
  tokenDisplay,
  hasTokenValue,
  formatTokenCount,
  formatRequestDate,
  formatCacheHitRate,
  REQUEST_RECORD_ROWS_PER_PAGE_OPTIONS,
} from 'src/sections/admin/request-records-utils';

export function UsageRecordsTable({
  rows,
  total,
  table,
  locale,
  loading,
}: {
  rows: UsageRecord[];
  total: number;
  table: UseTableReturn;
  locale: string;
  loading: boolean;
}) {
  const { t } = useTranslate('admin');
  const head = tableHead(t);
  const durationNow = useRequestRecordDurationNow(rows);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1180 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? <TableLoadingRows head={head} rows={table.rowsPerPage} /> : null}
            {!loading
              ? rows.map((row, index) => (
                  <UsageRecordRow
                    key={usageRecordKey(row, index)}
                    row={row}
                    locale={locale}
                    durationNow={durationNow}
                  />
                ))
              : null}
            <TableNoData title={t('common.noData')} notFound={!loading && rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        rowsPerPageOptions={REQUEST_RECORD_ROWS_PER_PAGE_OPTIONS}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function UsageRecordRow({
  row,
  locale,
  durationNow,
}: {
  row: UsageRecord;
  locale: string;
  durationNow: number;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatRequestDate(row.created_at, locale)}
      </TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 150 }}>
          {tokenDisplay(row)}
        </Typography>
      </TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 220 }}>
          {row.model_name || '-'}
        </Typography>
      </TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 180 }}>
          {formatApiFormat(row.client_api_format)}
        </Typography>
      </TableCell>
      <TableCell>
        <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
          <Label color="default" variant="soft">
            {row.is_stream ? t('requestRecords.stream') : t('requestRecords.nonStream')}
          </Label>
        </Stack>
      </TableCell>
      <TableCell align="right">
        <UsageTokensCell record={row} />
      </TableCell>
      <TableCell align="right">{formatCacheHitRate(row)}</TableCell>
      <TableCell>{formatCost(row.total_cost)}</TableCell>
      <TableCell>
        <RequestRecordDurationText record={row} metric="first_token" now={durationNow} />
      </TableCell>
      <TableCell>
        <RequestRecordDurationText record={row} metric="total_latency" now={durationNow} />
      </TableCell>
    </TableRow>
  );
}

function usageRecordKey(row: UsageRecord, index: number) {
  return [
    row.created_at,
    row.token_prefix ?? row.token_name ?? '',
    row.model_name ?? '',
    row.client_api_format,
    row.request_type,
    String(index),
  ].join(':');
}

function UsageTokensCell({ record }: { record: UsageRecord }) {
  const cacheCreation = hasTokenValue(record.cache_creation_input_tokens);
  const cacheRead = hasTokenValue(record.cache_read_input_tokens);

  return (
    <Stack alignItems="flex-end" spacing={0.5} sx={{ minWidth: 88 }}>
      <Stack direction="row" alignItems="center" spacing={0.75}>
        <Typography variant="caption">{formatTokenCount(record.prompt_tokens)}</Typography>
        <Typography variant="caption" color="text.secondary">
          /
        </Typography>
        <Typography variant="caption">{formatTokenCount(record.completion_tokens)}</Typography>
      </Stack>
      <Stack direction="row" alignItems="center" spacing={0.75}>
        <Typography
          variant="caption"
          color={cacheCreation ? 'text.primary' : 'text.secondary'}
          sx={cacheCreation ? activeCacheTokenSx : undefined}
        >
          {cacheCreation ? formatTokenCount(record.cache_creation_input_tokens) : '-'}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          /
        </Typography>
        <Typography
          variant="caption"
          color={cacheRead ? 'text.primary' : 'text.secondary'}
          sx={cacheRead ? activeCacheTokenSx : undefined}
        >
          {cacheRead ? formatTokenCount(record.cache_read_input_tokens) : '-'}
        </Typography>
      </Stack>
    </Stack>
  );
}

function tableHead(t: (key: string) => string): TableHeadCellProps[] {
  return [
    { id: 'time', label: t('requestRecords.time'), width: 190 },
    { id: 'token', label: t('requestRecords.token'), width: 150 },
    { id: 'model', label: t('requestRecords.model'), width: 220 },
    { id: 'api_format', label: t('requestRecords.apiFormat'), width: 180 },
    { id: 'type', label: t('requestRecords.type'), width: 150 },
    { id: 'tokens', label: t('requestRecords.tokens'), width: 140, align: 'right' },
    { id: 'cache_hit_rate', label: t('requestRecords.cacheHitRate'), width: 120, align: 'right' },
    { id: 'cost', label: t('requestRecords.cost'), width: 120 },
    { id: 'first_token', label: t('requestRecords.firstToken'), width: 110 },
    { id: 'latency', label: t('requestRecords.totalLatency'), width: 120 },
  ];
}

const activeCacheTokenSx = { opacity: 0.7 };
