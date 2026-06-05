'use client';

import type { TFunction } from 'i18next';
import type { TableHeadCellProps } from 'src/components/table';
import type { AdminAffiliateUserSummary, AdminAffiliateRelationChange } from 'src/types/affiliate';

import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TableHeadCustom, TablePaginationCustom } from 'src/components/table';

import { formatDate } from './admin-affiliate-format';

type RelationChangeTableProps = {
  t: TFunction<'admin'>;
  locale: string;
  rows: AdminAffiliateRelationChange[];
  total: number;
  loading: boolean;
  page: number;
  rowsPerPage: number;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function RelationChangesTable(props: RelationChangeTableProps) {
  const head = relationChangeHead(props.t);
  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 1420 }}>
          <TableHeadCustom headCells={head} />
          <TableBody>
            {props.loading ? (
              <LoadingRows t={props.t} head={head} rows={props.rowsPerPage} />
            ) : (
              props.rows.map((row) => (
                <RelationChangeRow key={row.id} t={props.t} locale={props.locale} row={row} />
              ))
            )}
            <TableNoData
              title={props.t('adminAffiliates.empty.relationChanges')}
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

function RelationChangeRow({
  t,
  locale,
  row,
}: {
  t: TFunction<'admin'>;
  locale: string;
  row: AdminAffiliateRelationChange;
}) {
  return (
    <TableRow hover>
      <TableCell>{formatDate(row.created_at, locale)}</TableCell>
      <UserCell user={row.user} />
      <UserCell user={row.old_referrer} />
      <UserCell user={row.new_referrer} />
      <OperatorCell t={t} row={row} />
      <TableCell>
        <Typography variant="body2" sx={{ whiteSpace: 'pre-wrap' }}>
          {row.reason}
        </Typography>
      </TableCell>
      <TableCell>{row.id}</TableCell>
    </TableRow>
  );
}

function UserCell({ user }: { user: AdminAffiliateUserSummary | null }) {
  if (!user) return <TableCell>-</TableCell>;
  return (
    <TableCell>
      <Stack spacing={0.5}>
        <Typography variant="body2" sx={{ fontWeight: 600 }}>
          {user.username || user.id}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {user.email}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {user.affiliate_code}
        </Typography>
      </Stack>
    </TableCell>
  );
}

function OperatorCell({ t, row }: { t: TFunction<'admin'>; row: AdminAffiliateRelationChange }) {
  if (row.operator) return <UserCell user={row.operator} />;
  if (!row.operator_user_id) return <TableCell>{t('common.system')}</TableCell>;
  return <TableCell>{row.operator_user_id}</TableCell>;
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

function relationChangeHead(t: TFunction<'admin'>): TableHeadCellProps[] {
  return [
    { id: 'created_at', label: t('adminAffiliates.fields.createdAt'), width: 180 },
    { id: 'user', label: t('adminAffiliates.fields.changedUser'), width: 230 },
    { id: 'old_referrer', label: t('adminAffiliates.fields.oldReferrer'), width: 230 },
    { id: 'new_referrer', label: t('adminAffiliates.fields.newReferrer'), width: 230 },
    { id: 'operator', label: t('adminAffiliates.fields.operator'), width: 220 },
    { id: 'reason', label: t('adminAffiliates.fields.reason'), width: 260 },
    { id: 'id', label: t('adminAffiliates.fields.changeId'), width: 220 },
  ];
}
