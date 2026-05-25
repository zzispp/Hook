'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { TableHeadCellProps } from 'src/components/table';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom } from 'src/components/table';

import { ModelCopyButton } from './model-copy-button';
import { priceSummary, billingBadges, formatUsageCount } from './model-catalog-utils';

// ----------------------------------------------------------------------

type Props = {
  rows: GlobalModelResponse[];
  loading: boolean;
  onSelectRow: (row: GlobalModelResponse) => void;
};

export function ModelCatalogTable({ rows, loading, onSelectRow }: Props) {
  const { t } = useTranslate('admin');
  const tableHead: TableHeadCellProps[] = [
    { id: 'model', label: t('fields.model'), width: 320 },
    { id: 'pricing', label: t('models.pricing'), width: 180 },
    { id: 'usage', label: t('models.usageCount'), width: 140 },
    { id: 'status', label: t('common.status'), width: 120 },
  ];

  return (
    <Scrollbar>
      <Table sx={{ minWidth: 760 }}>
        <TableHeadCustom headCells={tableHead} />
        <TableBody>
          {loading ? (
            <LoadingRows head={tableHead} />
          ) : (
            rows.map((row) => (
              <CatalogRow
                key={row.id}
                row={row}
                onSelectRow={onSelectRow}
              />
            ))
          )}
          <TableNoData title={t('models.emptyCatalog')} notFound={!loading && rows.length === 0} />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function CatalogRow({
  row,
  onSelectRow,
}: {
  row: GlobalModelResponse;
  onSelectRow: (row: GlobalModelResponse) => void;
}) {
  const { t } = useTranslate('admin');
  const badges = billingBadges(row);

  return (
    <TableRow hover sx={{ cursor: 'pointer' }} onClick={() => onSelectRow(row)}>
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
        <Typography variant="body2">{priceSummary(row)}</Typography>
        <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.5, my: 0.5 }}>
          {badges.map((badge) => (
            <Label key={badge} color={billingBadgeColor(badge)} variant="soft">
              {t(`models.${badge}`)}
            </Label>
          ))}
        </Stack>
        <Typography variant="caption" color="text.secondary">
          {t('models.inputOutputPrice')}
        </Typography>
      </TableCell>
      <TableCell>
        <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
          {formatUsageCount(row.usage_count)}
        </Typography>
      </TableCell>
      <TableCell>
        <Label color={row.is_active ? 'success' : 'default'} variant="soft">
          {row.is_active ? t('models.available') : t('models.unavailable')}
        </Label>
      </TableCell>
    </TableRow>
  );
}

function LoadingRows({ head }: { head: TableHeadCellProps[] }) {
  const { t } = useTranslate('admin');

  return Array.from({ length: 5 }).map((_, index) => (
    <TableRow key={index}>
      {head.map((cell) => (
        <TableCell key={cell.id || cell.label?.toString()} sx={{ color: 'text.disabled' }}>
          {t('common.loading')}
        </TableCell>
      ))}
    </TableRow>
  ));
}

function billingBadgeColor(badge: ReturnType<typeof billingBadges>[number]) {
  if (badge === 'billingMetered') return 'info';
  if (badge === 'billingTiered') return 'warning';
  return 'secondary';
}
