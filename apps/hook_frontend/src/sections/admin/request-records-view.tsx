'use client';

import type { RequestRecord } from 'src/types/provider';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRequestRecords, fetchActiveRequestRecords } from 'src/actions/request-records';

import { useTable } from 'src/components/table';
import { Iconify } from 'src/components/iconify';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { RequestRecordsTable } from './request-records-table';
import { RequestRecordDetailDrawer } from './request-record-detail-drawer';
import { requestStatusLabel, REQUEST_RECORD_STATUS_OPTIONS } from './request-records-utils';

const AUTO_REFRESH_INTERVAL_MS = 3000;
const ACTIVE_REQUEST_REFRESH_INTERVAL_MS = 1000;
const REQUEST_STATUS_RANK: Record<string, number> = {
  pending: 0,
  streaming: 1,
  success: 2,
  failed: 2,
};

type Filters = {
  search: string;
  status: string;
};

const DEFAULT_FILTERS: Filters = {
  search: '',
  status: '',
};

export function RequestRecordsView() {
  const { currentLang } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 20, defaultOrderBy: 'created_at' });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [selectedRecord, setSelectedRecord] = useState<RequestRecord | null>(null);
  const records = useRequestRecords(table.page, table.rowsPerPage, toQueryFilters(filters));
  const locale = currentLang.numberFormat.code;
  const refreshRecords = records.refresh;
  const activeRequestIds = useMemo(() => activeRecordIds(records.items), [records.items]);

  useAutoRefresh(autoRefresh, refreshRecords);
  useActiveRequestPolling(activeRequestIds, records.updateItems, refreshRecords);

  const handleFiltersChange = useCallback(
    (nextFilters: Filters) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.requestRecords}
        action={<RefreshButton loading={records.isValidating} onClick={refreshRecords} />}
      />
      <Card>
        <RequestRecordsToolbar
          filters={filters}
          autoRefresh={autoRefresh}
          loading={records.isValidating}
          onChange={handleFiltersChange}
          onRefresh={refreshRecords}
          onAutoRefreshChange={setAutoRefresh}
        />
        <RequestRecordsTable
          rows={records.items}
          total={records.total}
          table={table}
          locale={locale}
          loading={records.isLoading}
          onOpen={setSelectedRecord}
        />
      </Card>
      <RequestRecordDetailDrawer
        open={Boolean(selectedRecord)}
        record={selectedRecord}
        locale={locale}
        onClose={() => setSelectedRecord(null)}
      />
    </DashboardContent>
  );
}

function RequestRecordsToolbar({
  filters,
  autoRefresh,
  loading,
  onChange,
  onRefresh,
  onAutoRefreshChange,
}: {
  filters: Filters;
  autoRefresh: boolean;
  loading: boolean;
  onChange: (filters: Filters) => void;
  onRefresh: VoidFunction;
  onAutoRefreshChange: (value: boolean) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <TextField
        fullWidth
        value={filters.search}
        placeholder={t('filters.searchRequestRecords')}
        onChange={(event) => onChange({ ...filters, search: event.target.value })}
        slotProps={{
          input: {
            startAdornment: (
              <InputAdornment position="start">
                <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
              </InputAdornment>
            ),
          },
        }}
      />
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 180 }}
        onChange={(event) => onChange({ ...filters, status: event.target.value })}
      >
        <MenuItem value="">{t('filters.allStatuses')}</MenuItem>
        {REQUEST_RECORD_STATUS_OPTIONS.map((status) => (
          <MenuItem key={status} value={status}>
            {requestStatusLabel(status, t)}
          </MenuItem>
        ))}
      </TextField>
      <Stack direction="row" spacing={1.5} alignItems="center">
        <Button
          color="inherit"
          variant="outlined"
          loading={loading}
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
        >
          {t('common.refresh')}
        </Button>
        <FormControlLabel
          control={<Switch checked={autoRefresh} onChange={(event) => onAutoRefreshChange(event.target.checked)} />}
          label={<Typography variant="body2">{t('requestRecords.autoRefresh')}</Typography>}
          sx={{ whiteSpace: 'nowrap' }}
        />
      </Stack>
    </Stack>
  );
}

function useAutoRefresh(enabled: boolean, refresh: VoidFunction) {
  useEffect(() => {
    if (!enabled) return undefined;
    refresh();
    const timer = window.setInterval(refresh, AUTO_REFRESH_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [enabled, refresh]);
}

function useActiveRequestPolling(
  ids: string[],
  updateItems: (updater: (items: RequestRecord[]) => RequestRecord[]) => void,
  refresh: VoidFunction
) {
  const idsKey = ids.join('\n');

  useEffect(() => {
    if (!ids.length) return undefined;
    let inFlight = false;
    const poll = async () => {
      if (inFlight) return;
      inFlight = true;
      try {
        const response = await fetchActiveRequestRecords(ids);
        updateItems((items) => mergeRequestRecords(items, response.records));
        if (shouldRefreshRecords(ids, response.records)) refresh();
      } finally {
        inFlight = false;
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), ACTIVE_REQUEST_REFRESH_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [ids, idsKey, refresh, updateItems]);
}

function activeRecordIds(items: RequestRecord[]) {
  return items.filter((record) => isActiveRecord(record)).map((record) => record.request_id);
}

function isActiveRecord(record: RequestRecord) {
  return record.status === 'pending' || record.status === 'streaming';
}

function mergeRequestRecords(items: RequestRecord[], updates: RequestRecord[]) {
  if (!updates.length) return items;
  const updatesById = new Map(updates.map((record) => [record.request_id, record]));
  return items.map((item) => mergeRequestRecord(item, updatesById.get(item.request_id)));
}

function mergeRequestRecord(item: RequestRecord, update?: RequestRecord) {
  if (!update) return item;
  if (statusRank(update.status) < statusRank(item.status)) return item;
  return update;
}

function shouldRefreshRecords(ids: string[], updates: RequestRecord[]) {
  const updatedIds = new Set(updates.map((record) => record.request_id));
  return updates.some((record) => !isActiveRecord(record)) || ids.some((id) => !updatedIds.has(id));
}

function statusRank(status: string) {
  return REQUEST_STATUS_RANK[status] ?? 0;
}

function toQueryFilters(filters: Filters) {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
  };
}
