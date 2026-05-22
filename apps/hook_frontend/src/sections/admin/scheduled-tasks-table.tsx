'use client';

import type { Translate } from './scheduled-tasks-utils';
import type { ScheduledTask, ScheduledTaskRun } from 'src/types/scheduler';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TablePaginationCustom,
} from 'src/components/table';

import { formatWalletDateTime } from '../wallet/wallet-display';
import {
  EnabledLabel,
  TableLoadingRows,
  ManagementTableHead,
} from './shared';
import {
  taskLabel,
  formatTaskDuration,
  formatOptionalDate,
  translateTaskStatus,
} from './scheduled-tasks-utils';

export function ScheduledTaskTable({
  loading,
  rows,
  total,
  locale,
  table,
  onEdit,
  onToggle,
  t,
}: {
  loading: boolean;
  rows: ScheduledTask[];
  total: number;
  locale: string;
  table: UseTableReturn;
  onEdit: (task: ScheduledTask) => void;
  onToggle: (task: ScheduledTask, enabled: boolean) => void;
  t: Translate;
}) {
  const head = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'name', label: t('common.name'), width: 220 },
      { id: 'code', label: t('common.code'), width: 180 },
      { id: 'status', label: t('common.status'), width: 120 },
      { id: 'interval', label: t('scheduledTasks.fields.intervalSeconds'), width: 140 },
      { id: 'last_started_at', label: t('scheduledTasks.fields.lastStartedAt'), width: 180 },
      { id: 'last_finished_at', label: t('scheduledTasks.fields.lastFinishedAt'), width: 180 },
      { id: 'last_status', label: t('scheduledTasks.fields.lastStatus'), width: 140 },
      { id: 'last_duration_ms', label: t('scheduledTasks.fields.lastDuration'), width: 140 },
      { id: 'config', label: t('scheduledTasks.fields.config'), width: 260 },
      { id: '', width: 140 },
    ],
    [t]
  );

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1680 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? <TableLoadingRows head={head} rows={table.rowsPerPage} /> : null}
            {!loading
              ? rows.map((row) => (
                  <TableRow hover key={row.code}>
                    <TableCell>
                      <Stack spacing={0.5}>
                        <Typography variant="body2">{taskLabel(t, row)}</Typography>
                        <Typography variant="caption" color="text.secondary">
                          {t(row.description_key)}
                        </Typography>
                      </Stack>
                    </TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.enabled} />
                    </TableCell>
                    <TableCell>{row.interval_seconds}</TableCell>
                    <TableCell>{formatOptionalDate(row.last_started_at, locale)}</TableCell>
                    <TableCell>{formatOptionalDate(row.last_finished_at, locale)}</TableCell>
                    <TableCell>
                      <TaskStatusLabel status={row.last_status} t={t} />
                    </TableCell>
                    <TableCell>{formatTaskDuration(row.last_duration_ms)}</TableCell>
                    <TableCell>
                      <TaskConfigSummary task={row} t={t} />
                      {row.last_error ? (
                        <Alert severity="error" sx={{ mt: 1, py: 0 }}>
                          {row.last_error}
                        </Alert>
                      ) : null}
                    </TableCell>
                    <TableCell align="right">
                      <Stack direction="row" spacing={0.5} justifyContent="flex-end">
                        <Tooltip title={t('common.edit')}>
                          <IconButton onClick={() => onEdit(row)}>
                            <Iconify icon="solar:pen-bold" />
                          </IconButton>
                        </Tooltip>
                        <FormControlLabel
                          control={
                            <Switch
                              checked={row.enabled}
                              onChange={(event) => void onToggle(row, event.target.checked)}
                            />
                          }
                          label=""
                          sx={{ mr: 0 }}
                        />
                      </Stack>
                    </TableCell>
                  </TableRow>
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
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

export function ScheduledTaskRunTable({
  loading,
  rows,
  total,
  table,
  locale,
  tasks,
  t,
}: {
  loading: boolean;
  rows: ScheduledTaskRun[];
  total: number;
  table: UseTableReturn;
  locale: string;
  tasks: ScheduledTask[];
  t: Translate;
}) {
  const taskByCode = useMemo(
    () => new Map(tasks.map((task) => [task.code, task])),
    [tasks]
  );
  const head = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'task', label: t('scheduledTasks.fields.task'), width: 220 },
      { id: 'status', label: t('common.status'), width: 140 },
      { id: 'started_at', label: t('scheduledTasks.fields.startedAt'), width: 180 },
      { id: 'finished_at', label: t('scheduledTasks.fields.finishedAt'), width: 180 },
      { id: 'duration_ms', label: t('scheduledTasks.fields.duration'), width: 140 },
      { id: 'message', label: t('scheduledTasks.fields.result'), width: 260 },
      { id: 'error', label: t('scheduledTasks.fields.error'), width: 320 },
    ],
    [t]
  );

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1440 }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? <TableLoadingRows head={head} rows={table.rowsPerPage} /> : null}
            {!loading
              ? rows.map((row) => (
                  <TableRow hover key={row.id}>
                    <TableCell>
                      <Stack spacing={0.5}>
                        <Typography variant="body2">
                          {taskByCode.get(row.task_code) ? taskLabel(t, taskByCode.get(row.task_code) as ScheduledTask) : row.task_code}
                        </Typography>
                        <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
                          {row.task_code}
                        </Typography>
                      </Stack>
                    </TableCell>
                    <TableCell>
                      <TaskStatusLabel status={row.status} t={t} />
                    </TableCell>
                    <TableCell>{formatWalletDateTime(row.started_at, locale)}</TableCell>
                    <TableCell>{formatOptionalDate(row.finished_at, locale)}</TableCell>
                    <TableCell>{formatTaskDuration(row.duration_ms)}</TableCell>
                    <TableCell>
                      <Typography variant="body2" color="text.secondary">
                        {row.message || '-'}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2" color={row.error ? 'error.main' : 'text.secondary'}>
                        {row.error || '-'}
                      </Typography>
                    </TableCell>
                  </TableRow>
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
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function TaskConfigSummary({
  task,
  t,
}: {
  task: ScheduledTask;
  t: Translate;
}) {
  if (!task.config_schema.length) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('scheduledTasks.emptyConfig')}
      </Typography>
    );
  }

  return (
    <Stack spacing={0.5}>
      {task.config_schema.map((field) => (
        <Typography key={field.key} variant="caption" color="text.secondary">
          {t(field.label_key)}: {String(task.config[field.key] ?? '-')}
        </Typography>
      ))}
    </Stack>
  );
}

function TaskStatusLabel({
  status,
  t,
}: {
  status?: string | null;
  t: Translate;
}) {
  if (!status) {
    return (
      <Label variant="soft" color="default">
        {t('common.none')}
      </Label>
    );
  }

  const color =
    (status === 'succeeded' && 'success') ||
    (status === 'running' && 'info') ||
    (status === 'failed' && 'error') ||
    (status === 'skipped_running' && 'warning') ||
    'default';

  return (
    <Label color={color} variant="soft">
      {translateTaskStatus(t, status)}
    </Label>
  );
}
