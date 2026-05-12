'use client';

import type { RequestRecord } from 'src/types/provider';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useSystemSettings, useUsdCnyExchangeRate } from 'src/actions/system-settings';
import { useRequestRecords, fetchActiveRequestRecords } from 'src/actions/request-records';

import { useTable } from 'src/components/table';
import { Iconify } from 'src/components/iconify';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { RequestRecordsTable } from './request-records-table';
import { RequestRecordDetailDrawer } from './request-record-detail-drawer';
import {
  requestStatusLabel,
  REQUEST_RECORD_STATUS_OPTIONS,
  DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE,
} from './request-records-utils';

const AUTO_REFRESH_INTERVAL_MS = 3000;
const ACTIVE_REQUEST_REFRESH_INTERVAL_MS = 1000;
const ALL_STATUS_FILTER_VALUE = 'all';
const EMPTY_REQUEST_IDS: string[] = [];
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
  const { t, currentLang } = useTranslate('admin');
  const table = useTable({
    defaultRowsPerPage: DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE,
    defaultOrderBy: 'created_at',
  });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [manualRefreshing, setManualRefreshing] = useState(false);
  const [selectedRecord, setSelectedRecord] = useState<RequestRecord | null>(null);
  const settings = useSystemSettings();
  const exchangeRate = useUsdCnyExchangeRate(settings.data?.currency === 'CNY');
  const records = useRequestRecords(table.page, table.rowsPerPage, toQueryFilters(filters));
  const locale = currentLang.numberFormat.code;
  const currencyDisplay: CurrencyDisplay = {
    currency: settings.data?.currency ?? 'USD',
    usdCnyRate: exchangeRate.data,
    unavailableLabel: t('requestRecords.exchangeRateUnavailable'),
  };
  const refreshRecords = records.refresh;
  const refreshInFlightRef = useRef<Promise<unknown> | null>(null);
  const scrollSnapshotRef = useRef<number | null>(null);
  const pageVisible = usePageVisible();
  const activeRequestIds = useMemo(() => activeRecordIds(records.items), [records.items]);
  const pollingRequestIds = pageVisible ? activeRequestIds : EMPTY_REQUEST_IDS;
  const displaySelectedRecord = useMemo(
    () => latestSelectedRecord(selectedRecord, records.items),
    [records.items, selectedRecord]
  );

  const backgroundRefresh = useCallback(() => {
    if (refreshInFlightRef.current) return refreshInFlightRef.current;
    const next = Promise.resolve(refreshRecords()).finally(() => {
      if (refreshInFlightRef.current === next) refreshInFlightRef.current = null;
    });
    refreshInFlightRef.current = next;
    return next;
  }, [refreshRecords]);

  const handleManualRefresh = useCallback(async () => {
    setManualRefreshing(true);
    try {
      await backgroundRefresh();
    } finally {
      setManualRefreshing(false);
    }
  }, [backgroundRefresh]);

  const handleOpenRecord = useCallback((record: RequestRecord) => {
    scrollSnapshotRef.current = window.scrollY;
    setSelectedRecord(record);
  }, []);

  useAutoRefresh(autoRefresh && pageVisible, backgroundRefresh);
  useActiveRequestPolling(pollingRequestIds, records.updateItems, backgroundRefresh);
  useRestoreScrollOnSelection(displaySelectedRecord, scrollSnapshotRef);

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
        action={
          <RefreshButton loading={manualRefreshing} onClick={() => void handleManualRefresh()} />
        }
      />
      <Card>
        {settings.data?.currency === 'CNY' && exchangeRate.error ? (
          <Alert severity="error" sx={{ m: 2.5, mb: 0 }}>
            {t('requestRecords.exchangeRateLoadFailed')}
          </Alert>
        ) : null}
        <RequestRecordsToolbar
          filters={filters}
          autoRefresh={autoRefresh}
          onChange={handleFiltersChange}
          onAutoRefreshChange={setAutoRefresh}
        />
        <RequestRecordsTable
          rows={records.items}
          total={records.total}
          table={table}
          locale={locale}
          currencyDisplay={currencyDisplay}
          loading={records.isLoading}
          onOpen={handleOpenRecord}
        />
      </Card>
      <RequestRecordDetailDrawer
        open={Boolean(displaySelectedRecord)}
        record={displaySelectedRecord}
        locale={locale}
        currencyDisplay={currencyDisplay}
        onClose={() => setSelectedRecord(null)}
      />
    </DashboardContent>
  );
}

function RequestRecordsToolbar({
  filters,
  autoRefresh,
  onChange,
  onAutoRefreshChange,
}: {
  filters: Filters;
  autoRefresh: boolean;
  onChange: (filters: Filters) => void;
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
        value={filters.status || ALL_STATUS_FILTER_VALUE}
        sx={{ minWidth: 180 }}
        onChange={(event) =>
          onChange({ ...filters, status: statusFilterValue(event.target.value) })
        }
      >
        <MenuItem value={ALL_STATUS_FILTER_VALUE}>{t('filters.allStatuses')}</MenuItem>
        {REQUEST_RECORD_STATUS_OPTIONS.map((status) => (
          <MenuItem key={status} value={status}>
            {requestStatusLabel(status, t)}
          </MenuItem>
        ))}
      </TextField>
      <Stack direction="row" spacing={1.5} alignItems="center">
        <FormControlLabel
          control={
            <Switch
              checked={autoRefresh}
              onChange={(event) => onAutoRefreshChange(event.target.checked)}
            />
          }
          label={<Typography variant="body2">{t('requestRecords.autoRefresh')}</Typography>}
          sx={{ whiteSpace: 'nowrap' }}
        />
      </Stack>
    </Stack>
  );
}

function statusFilterValue(value: string) {
  return value === ALL_STATUS_FILTER_VALUE ? '' : value;
}

function useAutoRefresh(enabled: boolean, refresh: () => void) {
  useEffect(() => {
    if (!enabled) return undefined;
    refresh();
    const timer = window.setInterval(refresh, AUTO_REFRESH_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [enabled, refresh]);
}

function usePageVisible() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const update = () => setVisible(!document.hidden);
    update();
    document.addEventListener('visibilitychange', update);
    return () => document.removeEventListener('visibilitychange', update);
  }, []);

  return visible;
}

function useRestoreScrollOnSelection(
  selectedRecord: RequestRecord | null,
  scrollSnapshotRef: React.RefObject<number | null>
) {
  useEffect(() => {
    const scrollY = scrollSnapshotRef.current;
    if (!selectedRecord || scrollY === null) return undefined;
    const frame = window.requestAnimationFrame(() => {
      window.scrollTo({ top: scrollY, left: window.scrollX, behavior: 'instant' });
      scrollSnapshotRef.current = null;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [scrollSnapshotRef, selectedRecord]);
}

function useActiveRequestPolling(
  ids: string[],
  updateItems: (updater: (items: RequestRecord[]) => RequestRecord[]) => void,
  refresh: () => void
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
  return (
    record.status === 'pending' ||
    record.status === 'streaming' ||
    record.billing_status === 'pending'
  );
}

function mergeRequestRecords(items: RequestRecord[], updates: RequestRecord[]) {
  if (!updates.length) return items;
  const updatesById = new Map(updates.map((record) => [record.request_id, record]));
  return items.map((item) => mergeRequestRecord(item, updatesById.get(item.request_id)));
}

function mergeRequestRecord(item: RequestRecord, update?: RequestRecord) {
  if (!update) return item;
  if (statusRank(update.status) < statusRank(item.status)) return item;
  return {
    ...item,
    ...update,
    provider_id: update.provider_id ?? item.provider_id,
    provider_name: update.provider_name ?? item.provider_name,
    provider_key_name: update.provider_key_name ?? item.provider_key_name,
    provider_key_preview: update.provider_key_preview ?? item.provider_key_preview,
    provider_api_format: update.provider_api_format ?? item.provider_api_format,
    first_byte_time_ms: update.first_byte_time_ms ?? item.first_byte_time_ms,
    total_latency_ms: update.total_latency_ms ?? item.total_latency_ms,
    prompt_tokens: update.prompt_tokens ?? item.prompt_tokens,
    completion_tokens: update.completion_tokens ?? item.completion_tokens,
    total_tokens: update.total_tokens ?? item.total_tokens,
    cache_creation_input_tokens:
      update.cache_creation_input_tokens ?? item.cache_creation_input_tokens,
    cache_read_input_tokens: update.cache_read_input_tokens ?? item.cache_read_input_tokens,
    has_failover: item.has_failover || update.has_failover,
    has_retry: item.has_retry || update.has_retry,
    candidate_count: Math.max(item.candidate_count, update.candidate_count),
  };
}

function shouldRefreshRecords(ids: string[], updates: RequestRecord[]) {
  const updatedIds = new Set(updates.map((record) => record.request_id));
  return updates.some((record) => !isActiveRecord(record)) || ids.some((id) => !updatedIds.has(id));
}

function statusRank(status: string) {
  return REQUEST_STATUS_RANK[status] ?? 0;
}

function latestSelectedRecord(selectedRecord: RequestRecord | null, items: RequestRecord[]) {
  if (!selectedRecord) return null;
  return items.find((record) => record.request_id === selectedRecord.request_id) ?? selectedRecord;
}

function toQueryFilters(filters: Filters) {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
  };
}
