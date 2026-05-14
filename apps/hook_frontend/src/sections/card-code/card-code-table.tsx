'use client';

import type { TFunction } from 'i18next';
import type { CardCode } from 'src/types/card-code';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import {
  formatCardCodeDate,
  cardCodeStatusColor,
  cardCodeStatusLabel,
  formatCardCodeAmount,
} from './card-code-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rows: CardCode[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  selected: string[];
  onSelectRow: (id: string) => void;
  onSelectAllRows: (checked: boolean, ids: string[]) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function CardCodeTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {props.t('adminCardCodes.summary.codes', { shown: props.rows.length, total: props.total })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1680 }}>
          <TableHeadCustom
            headCells={head}
            rowCount={props.rows.length}
            numSelected={props.selected.length}
            onSelectAllRows={(checked) => props.onSelectAllRows(checked, props.rows.map((row) => row.id))}
          />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading ? props.rows.map((row) => <CardCodeRow key={row.id} row={row} {...props} />) : null}
            <TableNoData title={props.t('adminCardCodes.empty.codes')} notFound={!props.loading && props.rows.length === 0} />
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

function CardCodeRow({
  t,
  row,
  locale,
  selected,
  onSelectRow,
}: Pick<Props, 't' | 'locale' | 'selected' | 'onSelectRow'> & { row: CardCode }) {
  const checked = selected.includes(row.id);

  return (
    <TableRow hover selected={checked}>
      <TableCell padding="checkbox">
        <Checkbox checked={checked} onClick={() => onSelectRow(row.id)} />
      </TableCell>
      <TableCell sx={{ maxWidth: 240 }}>
        <Stack spacing={0.5} sx={{ minWidth: 0 }}>
          <Stack direction="row" spacing={0.75} alignItems="center" sx={{ minWidth: 0 }}>
            <Typography variant="body2" noWrap title={row.code} sx={{ minWidth: 0, fontWeight: 600 }}>
              {row.code}
            </Typography>
            <Tooltip title={t('adminCardCodes.actions.copyCode')}>
              <IconButton size="small" onClick={() => void copyCardCode(row.code, t)} sx={{ flexShrink: 0 }}>
                <Iconify icon="solar:copy-bold" width={14} />
              </IconButton>
            </Tooltip>
          </Stack>
          <Typography variant="caption" color="text.secondary">
            {row.batch_no}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell sx={{ maxWidth: 180 }}>
        <Typography variant="body2" noWrap title={row.type_name}>
          {row.type_name}
        </Typography>
      </TableCell>
      <TableCell>{formatCardCodeAmount(row.recharge_amount, row.gift_amount, row.currency)}</TableCell>
      <TableCell>
        <Label color={cardCodeStatusColor(row.status)} variant="soft">
          {cardCodeStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell>{row.used_by_username || '-'}</TableCell>
      <TableCell>{row.used_ip || '-'}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatCardCodeDate(row.used_at, locale)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatCardCodeDate(row.expires_at, locale)}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatCardCodeDate(row.created_at, locale)}</TableCell>
      <TableCell>{row.created_ip || '-'}</TableCell>
      <TableCell>{row.created_by_username || '-'}</TableCell>
      <TableCell>{row.remark || '-'}</TableCell>
    </TableRow>
  );
}

function LoadingRows({
  t,
  rows,
  head,
}: {
  t: TFunction<'admin'>;
  rows: number;
  head: TableHeadCellProps[];
}) {
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
    { id: 'code', label: t('adminCardCodes.fields.code'), width: 220 },
    { id: 'type', label: t('adminCardCodes.fields.type'), width: 160 },
    { id: 'amount', label: t('adminCardCodes.fields.amount'), width: 190 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'used_by', label: t('adminCardCodes.fields.usedBy'), width: 140 },
    { id: 'used_ip', label: t('adminCardCodes.fields.usedIp'), width: 140 },
    { id: 'used_at', label: t('adminCardCodes.fields.usedAt'), width: 180 },
    { id: 'expires_at', label: t('adminCardCodes.fields.expiresAt'), width: 180 },
    { id: 'created_at', label: t('adminCardCodes.fields.createdAt'), width: 180 },
    { id: 'created_ip', label: t('adminCardCodes.fields.createdIp'), width: 140 },
    { id: 'created_by', label: t('adminCardCodes.fields.createdBy'), width: 140 },
    { id: 'remark', label: t('adminCardCodes.fields.remark'), width: 180 },
  ];
}

async function copyCardCode(code: string, t: TFunction<'admin'>) {
  try {
    await navigator.clipboard.writeText(code);
    toast.success(t('adminCardCodes.messages.codeCopied'));
  } catch {
    toast.error(t('messages.copyFailed'));
  }
}
