'use client';

import type { ApiToken } from 'src/types/api-token';
import type { DisplayCurrency } from 'src/types/system-setting';
import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { formatMoney } from 'src/utils/currency-format';
import { ACCOUNTING_CURRENCY } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { formatTime, formatInteger } from './api-token-management-utils';
import { EnabledLabel, TableLoadingRows, ManagementTableHead } from '../admin/shared';

type Props = {
  rows: ApiToken[];
  total: number;
  loading: boolean;
  table: UseTableReturn;
  showOwner?: boolean;
  currencyDisplay?: CurrencyDisplay;
  onCopy: (token: ApiToken) => void;
  onEdit: (token: ApiToken) => void;
  onToggle: (token: ApiToken) => void;
  onDelete: (token: ApiToken) => void;
};

export function ApiTokenTable(props: Props) {
  const { t } = useTranslate('admin');
  const tableHead = tokenTableHead(t, props.showOwner, props.currencyDisplay?.currency);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: props.showOwner ? 1320 : 1100 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {props.loading ? (
              <TableLoadingRows head={tableHead} rows={props.table.rowsPerPage} />
            ) : (
              props.rows.map((row) => <ApiTokenTableRow key={row.id} row={row} props={props} />)
            )}
            <TableNoData
              title={t('common.noData')}
              notFound={!props.loading && props.rows.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={props.table.page}
        count={props.total}
        rowsPerPage={props.table.rowsPerPage}
        onPageChange={props.table.onChangePage}
        onRowsPerPageChange={props.table.onChangeRowsPerPage}
      />
    </>
  );
}

function ApiTokenTableRow({ row, props }: { row: ApiToken; props: Props }) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <Typography variant="subtitle2">{row.name}</Typography>
      </TableCell>
      <TableCell>
        <KeyCell token={row} onCopy={props.onCopy} />
      </TableCell>
      {props.showOwner ? (
        <TableCell>
          <OwnerCell token={row} />
        </TableCell>
      ) : null}
      {props.showOwner ? <TableCell>{t(tokenTypeKey(row.token_type))}</TableCell> : null}
      <TableCell>{formatTokenCost(row.used_quota, props.currencyDisplay)}</TableCell>
      <TableCell>{formatInteger(row.request_count)}</TableCell>
      <TableCell>{rateLimitText(row, t)}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.is_active} />
      </TableCell>
      <TableCell>{formatTime(row.last_used_at)}</TableCell>
      <TableCell align="right">
        <RowActions
          row={row}
          onEdit={props.onEdit}
          onToggle={props.onToggle}
          onDelete={props.onDelete}
        />
      </TableCell>
    </TableRow>
  );
}

function KeyCell({ token, onCopy }: { token: ApiToken; onCopy: (token: ApiToken) => void }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
        {token.token_prefix}...
      </Typography>
      <Tooltip title={t('actions.copyApiKey')}>
        <IconButton size="small" onClick={() => onCopy(token)}>
          <Iconify icon="solar:copy-bold" width={16} />
        </IconButton>
      </Tooltip>
    </Stack>
  );
}

function OwnerCell({ token }: { token: ApiToken }) {
  const { t } = useTranslate('admin');
  const primary = token.owner?.username || token.owner?.email;
  const email = token.owner?.email;

  if (!primary) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('common.none')}
      </Typography>
    );
  }

  return (
    <Stack spacing={0.25} sx={{ minWidth: 0 }}>
      <Typography variant="subtitle2" noWrap>
        {primary}
      </Typography>
      {email && email !== primary ? (
        <Typography variant="caption" color="text.secondary" noWrap>
          {email}
        </Typography>
      ) : null}
    </Stack>
  );
}

function RowActions({
  row,
  onEdit,
  onToggle,
  onDelete,
}: {
  row: ApiToken;
  onEdit: (token: ApiToken) => void;
  onToggle: (token: ApiToken) => void;
  onDelete: (token: ApiToken) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('common.edit')}>
        <IconButton onClick={() => onEdit(row)}>
          <Iconify icon="solar:pen-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={row.is_active ? t('actions.disable') : t('actions.enable')}>
        <IconButton onClick={() => onToggle(row)}>
          <Iconify icon={row.is_active ? 'solar:stop-circle-bold' : 'solar:play-circle-bold'} />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <IconButton color="error" onClick={() => onDelete(row)}>
          <Iconify icon="solar:trash-bin-trash-bold" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function tokenTableHead(
  t: (key: string, options?: Record<string, string>) => string,
  showOwner?: boolean,
  currency?: DisplayCurrency
): TableHeadCellProps[] {
  const ownerColumns = showOwner
    ? [
        { id: 'owner', label: t('fields.owner'), width: 220 },
        { id: 'token_type', label: t('fields.tokenType'), width: 130 },
      ]
    : [];

  return [
    { id: 'name', label: t('fields.keyName'), width: 180 },
    { id: 'key', label: t('fields.apiKey'), width: 180 },
    ...ownerColumns,
    { id: 'used_quota', label: costColumnLabel(t, currency), width: 140 },
    { id: 'request_count', label: t('fields.requestCount'), width: 130 },
    { id: 'rate_limit_rpm', label: t('fields.rateLimitRpm'), width: 140 },
    { id: 'status', label: t('common.status'), width: 110 },
    { id: 'last_used_at', label: t('fields.lastUsedAt'), width: 180 },
    { id: '', width: 136 },
  ];
}

function costColumnLabel(
  t: (key: string, options?: Record<string, string>) => string,
  currency?: DisplayCurrency
) {
  return t('fields.costWithCurrency', { currency: currency ?? ACCOUNTING_CURRENCY });
}

function formatTokenCost(value: number, currencyDisplay?: CurrencyDisplay) {
  return formatMoney(value, currencyDisplay ?? { currency: ACCOUNTING_CURRENCY });
}

function tokenTypeKey(type: ApiToken['token_type']) {
  return type === 'independent' ? 'tokens.independentToken' : 'tokens.userToken';
}

function rateLimitText(token: ApiToken, t: (key: string) => string) {
  const value = token.rate_limit_rpm ?? 0;
  return value === 0 ? t('tokens.followSystem') : `${formatInteger(value)} ${t('tokens.rpm')}`;
}
