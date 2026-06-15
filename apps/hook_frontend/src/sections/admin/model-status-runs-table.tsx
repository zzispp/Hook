'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { ModelStatusRun, ModelStatusValue } from 'src/types/model-status';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableHeadCustom,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { StatusLabel } from '../model-status/model-status-label';
import { latencyLabel } from '../model-status/model-status-timeline';

type Props = {
  rows: ModelStatusRun[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  locale: string;
  detail: ModelStatusRun | null;
  t: TFunction<'admin'>;
  onDetail: (run: ModelStatusRun | null) => void;
  onPageChange: (event: unknown, page: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function ModelStatusRunsTable(props: Props) {
  const head = tableHead(props.t);
  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1180 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows head={head} rows={props.rowsPerPage} t={props.t} /> : null}
            {!props.loading ? props.rows.map((row) => <RunRow key={row.id} row={row} {...props} />) : null}
            <TableNoData title={props.t('modelStatusChecks.runs.empty')} notFound={!props.loading && props.rows.length === 0} />
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
      <RunDetailDialog run={props.detail} locale={props.locale} t={props.t} onClose={() => props.onDetail(null)} />
    </>
  );
}

function RunRow({
  row,
  t,
  locale,
  onDetail,
}: Pick<Props, 't' | 'locale' | 'onDetail'> & { row: ModelStatusRun }) {
  return (
    <TableRow hover>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatDate(row.checked_at, locale)}</TableCell>
      <TableCell>
        <Typography variant="subtitle2">{row.model_name}</Typography>
        <Typography variant="caption" color="text.secondary">
          {row.check_name}
        </Typography>
      </TableCell>
      <TableCell>{row.api_format}</TableCell>
      <TableCell>{row.api_token_name}</TableCell>
      <TableCell>
        <StatusLabel status={row.status} t={t} />
      </TableCell>
      <TableCell>{latencyLabel(row.latency_ms)}</TableCell>
      <TableCell>{row.status_code ? `HTTP ${row.status_code}` : '-'}</TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Tooltip title={t('common.details')}>
          <IconButton onClick={() => onDetail(row)}>
            <Iconify icon="solar:eye-bold" />
          </IconButton>
        </Tooltip>
      </TableCell>
    </TableRow>
  );
}

function RunDetailDialog({
  run,
  locale,
  t,
  onClose,
}: {
  run: ModelStatusRun | null;
  locale: string;
  t: TFunction<'admin'>;
  onClose: VoidFunction;
}) {
  if (!run) return null;
  return (
    <Dialog fullWidth maxWidth="sm" open onClose={onClose}>
      <DialogTitle>{t('modelStatusChecks.runs.detailTitle')}</DialogTitle>
      <DialogContent>
        <Stack spacing={1.5} sx={{ pt: 1 }}>
          <DetailLine label={t('modelStatusChecks.runs.checkedAt')} value={formatDate(run.checked_at, locale)} />
          <DetailLine label={t('modelStatusChecks.model')} value={run.model_name} />
          <DetailLine label={t('modelStatusChecks.apiFormat')} value={run.api_format} />
          <DetailLine label={t('modelStatusChecks.apiToken')} value={run.api_token_name} />
          <DetailLine label={t('modelStatusChecks.lastLatency')} value={latencyLabel(run.latency_ms)} />
          <DetailLine label={t('modelStatusChecks.runs.statusCode')} value={run.status_code ? String(run.status_code) : '-'} />
          <Stack spacing={0.75}>
            <Typography variant="caption" color="text.secondary">
              {t('modelStatusChecks.lastStatus')}
            </Typography>
            <StatusLabel status={run.status} t={t} />
          </Stack>
          <MessageBlock label={t('modelStatusChecks.lastMessage')} value={run.message} status={run.status} t={t} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.close')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DetailLine({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.25}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2">{value}</Typography>
    </Stack>
  );
}

function MessageBlock({
  label,
  value,
  status,
  t,
}: {
  label: string;
  value?: string | null;
  status: ModelStatusValue;
  t: TFunction<'admin'>;
}) {
  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography component="pre" variant="body2" sx={messageSx}>
        {value || t(`modelStatusChecks.runs.defaultMessage.${status}`)}
      </Typography>
    </Stack>
  );
}

function LoadingRows({ head, rows, t }: { head: TableHeadCellProps[]; rows: number; t: TFunction<'admin'> }) {
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
    { id: 'checked_at', label: t('modelStatusChecks.runs.checkedAt'), width: 190 },
    { id: 'model', label: t('modelStatusChecks.model'), width: 260 },
    { id: 'api_format', label: t('modelStatusChecks.apiFormat'), width: 160 },
    { id: 'token', label: t('modelStatusChecks.apiToken'), width: 180 },
    { id: 'status', label: t('common.status'), width: 140 },
    { id: 'latency', label: t('modelStatusChecks.lastLatency'), width: 120 },
    { id: 'status_code', label: t('modelStatusChecks.runs.statusCode'), width: 120 },
    withStickyActionHeadCell({
      id: 'actions',
      label: t('common.actions'),
      align: 'left',
      width: 96,
    }),
  ];
}

function formatDate(value: string, locale: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString(locale);
}

const messageSx = {
  m: 0,
  p: 1.5,
  maxHeight: 240,
  overflow: 'auto',
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
  borderRadius: 1,
  bgcolor: 'background.neutral',
};
