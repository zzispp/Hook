'use client';

import type { TFunction } from 'i18next';
import type { CardCodeType } from 'src/types/card-code';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import {
  formatCardCodeDate,
  cardCodeStatusLabel,
  cardCodeTypeStatusColor,
  cardCodeBalanceTypeLabel,
} from './card-code-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rows: CardCodeType[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onEdit: (item: CardCodeType) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function CardCodeTypeTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {props.t('adminCardCodes.summary.types', { shown: props.rows.length, total: props.total })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 900 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading ? props.rows.map((row) => <CardCodeTypeRow key={row.id} row={row} {...props} />) : null}
            <TableNoData title={props.t('adminCardCodes.empty.types')} notFound={!props.loading && props.rows.length === 0} />
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

function CardCodeTypeRow({
  t,
  row,
  locale,
  onEdit,
}: Pick<Props, 't' | 'locale' | 'onEdit'> & { row: CardCodeType }) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {row.name}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {row.remark || '-'}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{cardCodeBalanceTypeLabel(t, row.balance_type)}</TableCell>
      <TableCell>
        <Label color={cardCodeTypeStatusColor(row.status)} variant="soft">
          {cardCodeStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatCardCodeDate(row.updated_at, locale)}</TableCell>
      <TableCell align="right">
        <Button size="small" variant="contained" startIcon={<Iconify icon="solar:pen-bold" />} onClick={() => onEdit(row)}>
          {t('common.edit')}
        </Button>
      </TableCell>
    </TableRow>
  );
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

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'name', label: t('adminCardCodes.fields.typeName'), width: 260 },
    { id: 'balance_type', label: t('adminCardCodes.fields.balanceType'), width: 220 },
    { id: 'status', label: t('common.status'), width: 140 },
    { id: 'updated_at', label: t('adminCardCodes.fields.updatedAt'), width: 180 },
    { id: 'action', label: t('wallet.table.action'), width: 120, align: 'right' },
  ];
}
