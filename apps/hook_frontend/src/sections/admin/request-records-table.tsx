'use client';

import type { RequestRecord } from 'src/types/provider';
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

import { formatApiFormat } from './provider-management-utils';
import { TableLoadingRows, ManagementTableHead } from './shared';
import {
  formatCost,
  formatTokens,
  formatDuration,
  formatRequestDate,
  requestStatusColor,
  requestStatusLabel,
} from './request-records-utils';

export function RequestRecordsTable({
  rows,
  total,
  table,
  locale,
  loading,
  onOpen,
}: {
  rows: RequestRecord[];
  total: number;
  table: UseTableReturn;
  locale: string;
  loading: boolean;
  onOpen: (record: RequestRecord) => void;
}) {
  const { t } = useTranslate('admin');
  const head = tableHead(t);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1240 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? <TableLoadingRows head={head} rows={table.rowsPerPage} /> : null}
            {!loading ? rows.map((row) => <RequestRecordRow key={row.request_id} row={row} locale={locale} onOpen={onOpen} />) : null}
            <TableNoData title={t('common.noData')} notFound={!loading && rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function RequestRecordRow({
  row,
  locale,
  onOpen,
}: {
  row: RequestRecord;
  locale: string;
  onOpen: (record: RequestRecord) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover sx={{ cursor: 'pointer' }} onClick={() => onOpen(row)}>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatRequestDate(row.created_at, locale)}</TableCell>
      <TableCell>{row.username || '-'}</TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 180 }}>
          {row.model_name || row.global_model_id || '-'}
        </Typography>
      </TableCell>
      <TableCell>{row.provider_name || '-'}</TableCell>
      <TableCell>{formatApiFormat(row.client_api_format)}</TableCell>
      <TableCell>
        <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
          <Label color={requestStatusColor(row.status)} variant="soft">
            {requestStatusLabel(row.status, t)}
          </Label>
          <Label color="default" variant="soft">
            {row.is_stream ? t('requestRecords.stream') : t('requestRecords.nonStream')}
          </Label>
        </Stack>
      </TableCell>
      <TableCell>{formatTokens(row)}</TableCell>
      <TableCell>{formatCost(row.total_cost)}</TableCell>
      <TableCell>{formatDuration(row.first_byte_time_ms)}</TableCell>
      <TableCell>{formatDuration(row.total_latency_ms)}</TableCell>
    </TableRow>
  );
}

function tableHead(t: (key: string) => string): TableHeadCellProps[] {
  return [
    { id: 'time', label: t('requestRecords.time'), width: 190 },
    { id: 'user', label: t('requestRecords.user'), width: 140 },
    { id: 'model', label: t('requestRecords.model'), width: 180 },
    { id: 'provider', label: t('requestRecords.provider'), width: 160 },
    { id: 'api_format', label: t('requestRecords.apiFormat'), width: 150 },
    { id: 'type', label: t('requestRecords.type'), width: 180 },
    { id: 'tokens', label: t('requestRecords.tokens'), width: 110 },
    { id: 'cost', label: t('requestRecords.cost'), width: 120 },
    { id: 'first_byte', label: t('requestRecords.firstByte'), width: 110 },
    { id: 'latency', label: t('requestRecords.totalLatency'), width: 120 },
  ];
}
