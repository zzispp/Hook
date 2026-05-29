'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusCheck } from 'src/types/model-status';
import type { TableHeadCellProps } from 'src/components/table';
import type { ModelStatusCheckFormState } from './model-status-check-form';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { updateModelStatusCheck } from 'src/actions/model-status';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import { EnabledLabel } from './shared';
import { StatusLabel } from '../model-status/model-status-view';
import { latencyLabel } from '../model-status/model-status-timeline';
import { intervalLabel, modelStatusCheckFormFromRow } from './model-status-check-form';

type Props = {
  rows: ModelStatusCheck[];
  loading: boolean;
  selected: string[];
  t: TFunction<'admin'>;
  onEdit: (form: ModelStatusCheckFormState) => void;
  onDelete: (row: ModelStatusCheck) => void;
  onSelectRow: (id: string) => void;
  onSelectAllRows: (checked: boolean, ids: string[]) => void;
};

export function ModelStatusChecksTable(props: Props) {
  const head = tableHead(props.t);
  return (
    <Scrollbar>
      <Table sx={{ minWidth: 1040 }}>
        <TableHeadCustom
          headCells={head}
          rowCount={props.rows.length}
          numSelected={props.selected.length}
          onSelectAllRows={(checked) => props.onSelectAllRows(checked, props.rows.map((row) => row.id))}
        />
        <TableBody>
          {props.loading ? <LoadingRows head={head} rows={5} t={props.t} /> : null}
          {!props.loading ? props.rows.map((row) => <CheckRow key={row.id} row={row} {...props} />) : null}
          <TableNoData title={props.t('modelStatusChecks.empty')} notFound={!props.loading && props.rows.length === 0} />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function CheckRow({
  row,
  t,
  selected,
  onEdit,
  onDelete,
  onSelectRow,
}: Omit<Props, 'rows' | 'loading' | 'onSelectAllRows'> & { row: ModelStatusCheck }) {
  const checked = selected.includes(row.id);
  return (
    <TableRow hover selected={checked}>
      <TableCell padding="checkbox">
        <Checkbox checked={checked} onClick={() => onSelectRow(row.id)} />
      </TableCell>
      <TableCell>
        <Typography variant="subtitle2">{row.name}</Typography>
        <Typography variant="caption" color="text.secondary">
          {row.api_format}
        </Typography>
      </TableCell>
      <TableCell>{row.model_name}</TableCell>
      <TableCell>{row.api_token_name}</TableCell>
      <TableCell>{intervalLabel(row.interval_seconds)}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell>
        <Stack spacing={0.5}>
          <StatusLabel status={row.last_status} t={t} />
          <Typography variant="caption" color="text.secondary">
            {latencyLabel(row.last_latency_ms)}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell align="right">
        <Switch size="small" checked={row.enabled} onChange={(event) => void toggleEnabled(row, event.target.checked, t)} />
        <Tooltip title={t('common.edit')}>
          <IconButton onClick={() => onEdit(modelStatusCheckFormFromRow(row))}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </Tooltip>
        <Tooltip title={t('common.delete')}>
          <IconButton color="error" onClick={() => onDelete(row)}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </Tooltip>
      </TableCell>
    </TableRow>
  );
}

function LoadingRows({ head, rows, t }: { head: TableHeadCellProps[]; rows: number; t: TFunction<'admin'> }) {
  return (
    <>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <TableRow key={rowIndex}>
          <TableCell padding="checkbox" />
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
    { id: 'name', label: t('modelStatusChecks.name'), width: 260 },
    { id: 'model', label: t('modelStatusChecks.model') },
    { id: 'token', label: t('modelStatusChecks.apiToken') },
    { id: 'interval', label: t('modelStatusChecks.interval') },
    { id: 'enabled', label: t('modelStatusChecks.enabled') },
    { id: 'status', label: t('modelStatusChecks.lastStatus') },
    { id: 'actions', label: '', align: 'right' },
  ];
}

async function toggleEnabled(row: ModelStatusCheck, enabled: boolean, t: TFunction<'admin'>) {
  try {
    await updateModelStatusCheck(row.id, { enabled });
    toast.success(t('modelStatusChecks.messages.updated'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
}
