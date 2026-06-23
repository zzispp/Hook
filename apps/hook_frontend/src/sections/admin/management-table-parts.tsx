'use client';

import type { TableHeadCellProps } from 'src/components/table';

import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';

import { useTranslate } from 'src/locales/use-locales';

import { TableHeadCustom } from 'src/components/table';

export function TableLoadingRows({
  head,
  rows = 5,
}: {
  head: TableHeadCellProps[];
  rows?: number;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <TableRow key={rowIndex}>
          {head.map((cell) => (
            <TableCell
              key={cell.id || cell.label?.toString() || 'action'}
              sx={[
                { color: 'text.disabled' },
                ...(Array.isArray(cell.sx) ? cell.sx : [cell.sx]),
              ]}
            >
              {t('common.loading')}
            </TableCell>
          ))}
        </TableRow>
      ))}
    </>
  );
}

export function ManagementTableHead({ head }: { head: TableHeadCellProps[] }) {
  return <TableHeadCustom headCells={head} />;
}
