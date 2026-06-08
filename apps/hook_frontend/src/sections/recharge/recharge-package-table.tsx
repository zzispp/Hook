'use client';

import type { TFunction } from 'i18next';
import type { RechargePackage } from 'src/types/recharge';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TableHeadCustom,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import {
  formatCny,
  formatUsd,
  formatRechargeDate,
  packageStatusColor,
  estimatedPayableAmount,
  rechargePackageStatusLabel,
} from './recharge-display';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  ratio: number;
  rows: RechargePackage[];
  total: number;
  loading: boolean;
  busy: boolean;
  page: number;
  rowsPerPage: number;
  onEdit: (item: RechargePackage) => void;
  onToggleStatus: (item: RechargePackage) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RechargePackageTable(props: Props) {
  const head = tableHead(props.t);

  return (
    <>
      <Typography variant="body2" color="text.secondary" sx={{ px: 2.5, pb: 2 }}>
        {props.t('adminRecharges.summary.packages', {
          shown: props.rows.length,
          total: props.total,
        })}
      </Typography>
      <Scrollbar>
        <Table sx={{ minWidth: 1120 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} /> : null}
            {!props.loading
              ? props.rows.map((row) => <PackageRow key={row.id} row={row} {...props} />)
              : null}
            <TableNoData
              title={props.t('adminRecharges.empty.packages')}
              notFound={!props.loading && props.rows.length === 0}
            />
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

function PackageRow({
  t,
  row,
  locale,
  ratio,
  busy,
  onEdit,
  onToggleStatus,
}: Pick<Props, 't' | 'locale' | 'ratio' | 'busy' | 'onEdit' | 'onToggleStatus'> & {
  row: RechargePackage;
}) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {row.name}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {row.description || '-'}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>{formatUsd(row.recharge_amount)}</TableCell>
      <TableCell>{formatUsd(row.gift_amount)}</TableCell>
      <TableCell>{formatUsd(row.total_arrival_amount)}</TableCell>
      <TableCell>{formatCny(estimatedPayableAmount(row.recharge_amount, ratio))}</TableCell>
      <TableCell>
        <Label color={packageStatusColor(row.status)} variant="soft">
          {rechargePackageStatusLabel(t, row.status)}
        </Label>
      </TableCell>
      <TableCell>{row.sort_order}</TableCell>
      <TableCell sx={{ whiteSpace: 'nowrap' }}>{formatRechargeDate(row.updated_at, locale)}</TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Stack direction="row" justifyContent="flex-end" spacing={0.5}>
          <Tooltip title={t('common.edit')}>
            <span>
              <IconButton size="small" disabled={busy} onClick={() => onEdit(row)}>
                <Iconify icon="solar:pen-bold" />
              </IconButton>
            </span>
          </Tooltip>
          <Tooltip title={statusActionLabel(t, row.status)}>
            <span>
              <IconButton
                size="small"
                color={row.status === 'active' ? 'warning' : 'success'}
                disabled={busy}
                onClick={() => onToggleStatus(row)}
              >
                <Iconify icon={row.status === 'active' ? 'solar:stop-circle-bold' : 'solar:check-circle-bold'} />
              </IconButton>
            </span>
          </Tooltip>
        </Stack>
      </TableCell>
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

function statusActionLabel(t: TFunction<'admin'>, status: string) {
  return status === 'active'
    ? t('adminRecharges.actions.disablePackage')
    : t('adminRecharges.actions.enablePackage');
}

function tableHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'name', label: t('adminRecharges.fields.packageName'), width: 240 },
    { id: 'recharge_amount', label: t('adminRecharges.fields.rechargeAmount'), width: 130 },
    { id: 'gift_amount', label: t('adminRecharges.fields.giftAmount'), width: 130 },
    { id: 'total_arrival_amount', label: t('adminRecharges.fields.totalArrival'), width: 130 },
    { id: 'estimated_payable', label: t('adminRecharges.fields.estimatedPayable'), width: 140 },
    { id: 'status', label: t('common.status'), width: 120 },
    { id: 'sort_order', label: t('common.sortOrder'), width: 100 },
    { id: 'updated_at', label: t('adminRecharges.fields.updatedAt'), width: 180 },
    withStickyActionHeadCell({
      id: 'action',
      label: t('wallet.table.action'),
      width: 120,
      align: 'left',
    }),
  ];
}
