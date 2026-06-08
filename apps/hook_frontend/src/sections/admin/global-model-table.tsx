'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { formatMoneyCompact } from 'src/utils/currency-format';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import {
  TableNoData,
  TablePaginationCustom,
  tableStickyActionCellSx,
  withStickyActionHeadCell,
} from 'src/components/table';

import { ModelCopyButton } from '../models/model-copy-button';
import { formatUsageCount } from '../models/model-catalog-utils';
import { EnabledLabel, TableLoadingRows, ManagementTableHead } from './shared';
import { firstTier, formFromModel, capabilitiesFromForm } from './model-management-utils';

// ----------------------------------------------------------------------

type Props = {
  rows: GlobalModelResponse[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  onDetail: (model: GlobalModelResponse) => void;
  onEdit: (model: GlobalModelResponse) => void;
  onDelete: (model: GlobalModelResponse) => void;
};

export function GlobalModelTable({ rows, total, loading, table, onDetail, onEdit, onDelete }: Props) {
  const { t } = useTranslate('admin');
  const tableHead: TableHeadCellProps[] = [
    { id: 'name', label: t('fields.model'), width: 280 },
    { id: 'pricing', label: t('models.pricing'), width: 220 },
    { id: 'capabilities', label: t('models.capabilities') },
    { id: 'providers', label: t('models.providers'), width: 150 },
    { id: 'usage_count', label: t('models.usageCount'), width: 130 },
    { id: 'status', label: t('common.status'), width: 120 },
    withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 136, align: 'left' }),
  ];

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1080 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              rows.map((row) => (
                <GlobalModelTableRow
                  key={row.id}
                  row={row}
                  onDetail={onDetail}
                  onEdit={onEdit}
                  onDelete={onDelete}
                />
              ))
            )}

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

function GlobalModelTableRow({
  row,
  onDetail,
  onEdit,
  onDelete,
}: {
  row: GlobalModelResponse;
  onDetail: (model: GlobalModelResponse) => void;
  onEdit: (model: GlobalModelResponse) => void;
  onDelete: (model: GlobalModelResponse) => void;
}) {
  const { t } = useTranslate('admin');
  const tier = firstTier(row);
  const capabilities = capabilitiesFromForm(formFromModel(row));

  return (
    <TableRow hover>
      <TableCell>
        <Typography variant="subtitle2">{row.display_name}</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
            {row.name}
          </Typography>
          <ModelCopyButton value={row.name} />
        </Stack>
      </TableCell>
      <TableCell>
        <Stack spacing={0.25}>
          <Typography variant="body2">
            {formatPrice(tier?.input_price_per_1m)} / {formatPrice(tier?.output_price_per_1m)}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {t('models.inputOutputPrice')}
          </Typography>
        </Stack>
      </TableCell>
      <TableCell>
        <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
          {capabilities.slice(0, 5).map((capability) => (
            <Chip key={capability} size="small" label={capability} />
          ))}
          {capabilities.length > 5 && <Chip size="small" label={`+${capabilities.length - 5}`} />}
        </Stack>
      </TableCell>
      <TableCell>
        <Typography variant="body2">
          {row.active_provider_count ?? 0} / {row.provider_count ?? 0}
        </Typography>
      </TableCell>
      <TableCell>
        <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
          {formatUsageCount(row.usage_count)}
        </Typography>
      </TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell align="left" sx={tableStickyActionCellSx}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('models.detailTitle')}>
            <IconButton onClick={() => onDetail(row)}>
              <Iconify icon="solar:eye-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.edit')}>
            <IconButton onClick={() => onEdit(row)}>
              <Iconify icon="solar:pen-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <IconButton color="error" onClick={() => onDelete(row)}>
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function formatPrice(value: number | null | undefined) {
  if (value === null || value === undefined) return '-';
  return formatMoneyCompact(value);
}
