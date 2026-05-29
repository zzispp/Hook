'use client';

import type { TFunction } from 'i18next';
import type { AdminDashboardRangeFilters } from '../admin/dashboard-date-range-picker';
import type { ModelStatusCheck, ModelStatusValue, ModelStatusListFilters } from 'src/types/model-status';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import TableContainer from '@mui/material/TableContainer';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useModelStatusChecks } from 'src/actions/model-status';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { MODEL_STATUS_API_FORMATS } from './model-status-options';
import { DashboardDateRangePicker } from '../admin/dashboard-date-range-picker';
import { RefreshButton, AdminBreadcrumbs, TableLoadingRows } from '../admin/shared';
import { statusLabel, latencyLabel, ModelStatusTimeline } from './model-status-timeline';

const ALL_API_FORMATS = '';

const TABLE_HEAD = [
  { id: 'name', label: 'Name' },
  { id: 'status', label: 'Status' },
  { id: 'availability', label: 'Availability', align: 'right' as const },
  { id: 'timeline', label: 'Timeline' },
];

export function ModelStatusView() {
  const { t } = useTranslate('admin');
  const [filters, setFilters] = useState<ModelStatusListFilters>({ preset: 'today' });
  const records = useModelStatusChecks(filters);
  const changeRange = (range: AdminDashboardRangeFilters) =>
    setFilters((current) => ({ ...range, search: current.search, api_format: current.api_format }));

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.modelStatus}
        action={
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1}>
            <DashboardDateRangePicker
              t={t}
              filters={filters}
              translationRoot="modelStatus"
              onChange={changeRange}
            />
            <RefreshButton loading={records.isValidating} onClick={() => void records.refresh()} />
          </Stack>
        }
      />
      <Stack spacing={3}>
        {records.error ? <Alert severity="error">{errorMessage(records.error)}</Alert> : null}
        <Card>
          <ModelStatusToolbar filters={filters} t={t} onChange={setFilters} />
          <TableContainer component={Scrollbar}>
            <Table sx={{ minWidth: 860 }}>
              <TableHead>
                <TableRow>
                  {TABLE_HEAD.map((cell) => (
                    <TableCell key={cell.id} align={cell.align}>
                      {tableLabel(cell.id, t)}
                    </TableCell>
                  ))}
                </TableRow>
              </TableHead>
              <TableBody>
                {records.isLoading ? <TableLoadingRows head={TABLE_HEAD} rows={5} /> : null}
                {!records.isLoading && records.items.map((row) => <ModelStatusRow key={row.id} row={row} t={t} />)}
                {!records.isLoading && !records.items.length ? <EmptyRow t={t} /> : null}
              </TableBody>
            </Table>
          </TableContainer>
        </Card>
      </Stack>
    </DashboardContent>
  );
}

function ModelStatusToolbar({
  filters,
  t,
  onChange,
}: {
  filters: ModelStatusListFilters;
  t: TFunction<'admin'>;
  onChange: (filters: ModelStatusListFilters) => void;
}) {
  const patch = (patchFilters: Partial<ModelStatusListFilters>) => onChange({ ...filters, ...patchFilters });
  return (
    <Box sx={{ p: 2.5 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SearchField value={filters.search ?? ''} t={t} onChange={(search) => patch({ search })} />
        <ApiFormatFilter value={filters.api_format ?? ALL_API_FORMATS} t={t} onChange={(api_format) => patch({ api_format })} />
      </Stack>
    </Box>
  );
}

function SearchField({
  value,
  t,
  onChange,
}: {
  value: string;
  t: TFunction<'admin'>;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      fullWidth
      size="small"
      value={value}
      placeholder={t('modelStatus.searchPlaceholder')}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{ input: { startAdornment: <SearchIcon /> } }}
    />
  );
}

function ApiFormatFilter({
  value,
  t,
  onChange,
}: {
  value: string;
  t: TFunction<'admin'>;
  onChange: (value: string) => void;
}) {
  return (
    <TextField select size="small" label={t('modelStatus.apiFormat')} value={value} sx={{ minWidth: 180 }} onChange={(event) => onChange(event.target.value)}>
      <MenuItem value={ALL_API_FORMATS}>{t('modelStatus.allApiFormats')}</MenuItem>
      {MODEL_STATUS_API_FORMATS.map((format) => <MenuItem key={format} value={format}>{format}</MenuItem>)}
    </TextField>
  );
}

function SearchIcon() {
  return (
    <InputAdornment position="start">
      <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
    </InputAdornment>
  );
}

function ModelStatusRow({ row, t }: { row: ModelStatusCheck; t: TFunction<'admin'> }) {
  return (
    <TableRow hover>
      <TableCell>
        <Stack spacing={0.5}>
          <Typography variant="subtitle2">{row.name}</Typography>
          <Typography variant="caption" color="text.secondary">
            {row.model_name} · {row.api_format}
          </Typography>
        </Stack>
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
        <Typography variant="subtitle2">{availability(row)}</Typography>
        <Typography variant="caption" color="text.secondary">
          {row.availability.available_checks}/{row.availability.total_checks}
        </Typography>
      </TableCell>
      <TableCell sx={{ minWidth: 360 }}>
        <ModelStatusTimeline points={row.timeline} t={t} />
      </TableCell>
    </TableRow>
  );
}

export function StatusLabel({ status, t }: { status?: ModelStatusValue | null; t: TFunction<'admin'> }) {
  return (
    <Label color={statusColor(status)} variant="soft">
      {statusLabel(status, t)}
    </Label>
  );
}

function EmptyRow({ t }: { t: TFunction<'admin'> }) {
  return (
    <TableRow>
      <TableCell colSpan={TABLE_HEAD.length}>
        <Typography sx={{ py: 5, textAlign: 'center', color: 'text.secondary' }}>
          {t('modelStatus.empty')}
        </Typography>
      </TableCell>
    </TableRow>
  );
}

function tableLabel(id: string, t: TFunction<'admin'>) {
  if (id === 'name') return t('modelStatus.name');
  if (id === 'status') return t('modelStatus.status');
  if (id === 'availability') return t('modelStatus.availability');
  return t('modelStatus.timeline');
}

function availability(row: ModelStatusCheck) {
  return row.availability.availability_pct ? `${row.availability.availability_pct}%` : '-';
}

function statusColor(status?: ModelStatusValue | null) {
  if (status === 'operational') return 'success';
  if (status === 'degraded') return 'warning';
  if (status === 'failed' || status === 'error') return 'error';
  return 'default';
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Request failed';
}
