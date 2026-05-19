'use client';

import type { RequestRecord } from 'src/types/provider';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import SvgIcon from '@mui/material/SvgIcon';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { TableLoadingRows, ManagementTableHead } from './shared';
import {
  RequestRecordDurationText,
  useRequestRecordDurationNow,
} from './request-record-duration-text';
import {
  formatCost,
  userDisplay,
  hasTokenValue,
  formatTokenCount,
  formatRequestDate,
  requestStatusColor,
  requestStatusLabel,
  formatCacheHitRate,
  formatRequestApiFormat,
  REQUEST_RECORD_ROWS_PER_PAGE_OPTIONS,
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
  const durationNow = useRequestRecordDurationNow(rows);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1360 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? <TableLoadingRows head={head} rows={table.rowsPerPage} /> : null}
            {!loading
              ? rows.map((row) => (
                  <RequestRecordRow
                    key={row.request_id}
                    row={row}
                    locale={locale}
                    durationNow={durationNow}
                    onOpen={onOpen}
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

function RequestRecordRow({
  row,
  locale,
  durationNow,
  onOpen,
}: {
  row: RequestRecord;
  locale: string;
  durationNow: number;
  onOpen: (record: RequestRecord) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover sx={{ cursor: 'pointer' }} onClick={() => onOpen(row)}>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>
        {formatRequestDate(row.created_at, locale)}
      </TableCell>
      <TableCell>{userDisplay(row)}</TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 180 }}>
          {row.model_name || row.global_model_id || '-'}
        </Typography>
      </TableCell>
      <TableCell>
        <ProviderCell record={row} />
      </TableCell>
      <TableCell>
        <Typography variant="body2" noWrap sx={{ maxWidth: 240 }}>
          {formatRequestApiFormat(row)}
        </Typography>
      </TableCell>
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
      <TableCell align="right">
        <RequestTokensCell record={row} />
      </TableCell>
      <TableCell align="right">{formatCacheHitRate(row)}</TableCell>
      <TableCell>{formatCost(row.total_cost)}</TableCell>
      <TableCell>
        <RequestRecordDurationText record={row} metric="first_byte" now={durationNow} />
      </TableCell>
      <TableCell>
        <RequestRecordDurationText record={row} metric="total_latency" now={durationNow} />
      </TableCell>
    </TableRow>
  );
}

function ProviderCell({ record }: { record: RequestRecord }) {
  const { t } = useTranslate('admin');
  const keyLabel = providerKeyLabel(record);

  return (
    <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
      <Stack spacing={0.25} sx={{ minWidth: 0 }}>
        <Typography variant="caption" noWrap sx={{ maxWidth: 160 }}>
          {record.provider_name || '-'}
        </Typography>
        {keyLabel ? (
          <Typography
            variant="caption"
            color="text.secondary"
            noWrap
            title={keyLabel}
            sx={{ maxWidth: 160 }}
          >
            {keyLabel}
          </Typography>
        ) : null}
      </Stack>
      <ProviderExecutionIcon record={record} t={t} />
    </Stack>
  );
}

function ProviderExecutionIcon({
  record,
  t,
}: {
  record: RequestRecord;
  t: (key: string) => string;
}) {
  if (record.has_failover) {
    return (
      <Tooltip title={t('requestRecords.providerFailoverTooltip')}>
        <SvgIcon sx={failoverIconSx}>
          <path d="m16 3 4 4-4 4" />
          <path d="M20 7H4" />
          <path d="m8 21-4-4 4-4" />
          <path d="M4 17h16" />
        </SvgIcon>
      </Tooltip>
    );
  }
  if (!record.has_retry) return null;

  return (
    <Tooltip title={t('requestRecords.providerRetryTooltip')}>
      <SvgIcon sx={retryIconSx}>
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
        <path d="M21 21v-5h-5" />
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
        <path d="M3 3v5h5" />
      </SvgIcon>
    </Tooltip>
  );
}

function providerKeyLabel(record: RequestRecord) {
  return record.provider_key_name || record.provider_key_preview || '';
}

function RequestTokensCell({ record }: { record: RequestRecord }) {
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
    { id: 'user', label: t('requestRecords.user'), width: 140 },
    { id: 'model', label: t('requestRecords.model'), width: 180 },
    { id: 'provider', label: t('requestRecords.provider'), width: 190 },
    { id: 'api_format', label: t('requestRecords.apiFormat'), width: 240 },
    { id: 'type', label: t('requestRecords.type'), width: 180 },
    { id: 'tokens', label: t('requestRecords.tokens'), width: 140, align: 'right' },
    { id: 'cache_hit_rate', label: t('requestRecords.cacheHitRate'), width: 120, align: 'right' },
    { id: 'cost', label: t('requestRecords.cost'), width: 120 },
    { id: 'first_byte', label: t('requestRecords.firstByte'), width: 110 },
    { id: 'latency', label: t('requestRecords.totalLatency'), width: 120 },
  ];
}

const activeCacheTokenSx = { opacity: 0.7 };

const providerIconBaseSx = {
  width: 14,
  height: 14,
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 2,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
  flexShrink: 0,
};

const failoverIconSx = {
  ...providerIconBaseSx,
  color: 'warning.dark',
};

const retryIconSx = {
  ...providerIconBaseSx,
  color: 'info.dark',
};
