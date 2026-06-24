'use client';

import type { RequestRecord } from 'src/types/provider';

import { useRef, useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';

import { useProviders } from 'src/actions/providers';
import { useGlobalModels } from 'src/actions/models';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useRequestRecords } from 'src/actions/request-records';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { RequestRecordsTable } from './request-records-table';
import { RequestRecordDetailDrawer } from './request-record-detail-drawer';
import { DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE } from './request-records-utils';
import {
  usePageVisible,
  useAutoRefresh,
  usePollingRequestIds,
  useActiveRequestPolling,
  useRestoreScrollOnSelection,
} from './request-records-polling';
import {
  RequestRecordsToolbar,
  toRequestRecordQueryFilters,
  type RequestRecordFilterState,
  DEFAULT_REQUEST_RECORD_FILTERS,
} from './request-records-toolbar';

const AUTO_REFRESH_INTERVAL_MS = 3000;

export function RequestRecordsView() {
  return <RequestRecordsContent {...useRequestRecordsViewProps()} />;
}

function useRequestRecordsViewProps() {
  const { currentLang } = useTranslate('admin');
  const table = useTable({
    defaultRowsPerPage: DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE,
    defaultOrderBy: 'created_at',
  });
  const { filters, handleFiltersChange } = useRequestRecordFilters(table.onResetPage);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [timingExpanded, setTimingExpanded] = useState(false);
  const models = useGlobalModels(0, 1000);
  const providers = useProviders(0, 1000);
  const records = useRequestRecords(
    table.page,
    table.rowsPerPage,
    toRequestRecordQueryFilters(filters)
  );
  const locale = currentLang.numberFormat.code;
  const pageVisible = usePageVisible();
  const pollingRequestIds = usePollingRequestIds(records.items, pageVisible);
  const refresh = useRequestRecordRefresh(records.refresh);
  const selection = useRequestRecordSelection(records.items);

  useAutoRefresh(autoRefresh && pageVisible, refresh.backgroundRefresh, AUTO_REFRESH_INTERVAL_MS);
  useActiveRequestPolling(
    pollingRequestIds,
    filters.status,
    records.updateItems,
    refresh.backgroundRefresh
  );
  useRestoreScrollOnSelection(selection.displaySelectedRecord, selection.scrollSnapshotRef);

  return {
    table,
    filters,
    models: models.items,
    records,
    providers: providers.items,
    autoRefresh,
    timingExpanded,
    manualRefreshing: refresh.manualRefreshing,
    displaySelectedRecord: selection.displaySelectedRecord,
    locale,
    onOpenRecord: selection.handleOpenRecord,
    onFiltersChange: handleFiltersChange,
    onAutoRefreshChange: setAutoRefresh,
    onTimingExpandedChange: setTimingExpanded,
    onManualRefresh: refresh.handleManualRefresh,
    onCloseRecord: selection.handleCloseRecord,
  };
}

function useRequestRecordFilters(resetPage: () => void) {
  const [filters, setFilters] = useState(DEFAULT_REQUEST_RECORD_FILTERS);
  const handleFiltersChange = useCallback(
    (nextFilters: RequestRecordFilterState) => {
      resetPage();
      setFilters(nextFilters);
    },
    [resetPage]
  );
  return { filters, handleFiltersChange };
}

function useRequestRecordRefresh(refreshRecords: () => Promise<unknown> | unknown) {
  const [manualRefreshing, setManualRefreshing] = useState(false);
  const refreshInFlightRef = useRef<Promise<unknown> | null>(null);
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
  return { manualRefreshing, backgroundRefresh, handleManualRefresh };
}

function useRequestRecordSelection(items: RequestRecord[]) {
  const [selectedRecord, setSelectedRecord] = useState<RequestRecord | null>(null);
  const scrollSnapshotRef = useRef<number | null>(null);
  const displaySelectedRecord = useMemo(
    () => latestSelectedRecord(selectedRecord, items),
    [items, selectedRecord]
  );
  const handleOpenRecord = useCallback((record: RequestRecord) => {
    scrollSnapshotRef.current = window.scrollY;
    setSelectedRecord(record);
  }, []);
  const handleCloseRecord = useCallback(() => setSelectedRecord(null), []);
  return { displaySelectedRecord, scrollSnapshotRef, handleOpenRecord, handleCloseRecord };
}

type RequestRecordsContentProps = ReturnType<typeof useRequestRecordsViewProps>;

function RequestRecordsContent(props: RequestRecordsContentProps) {
  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.requestRecords}
        action={
          <RefreshButton
            loading={props.manualRefreshing}
            onClick={() => void props.onManualRefresh()}
          />
        }
      />
      <Card>
        <RequestRecordsToolbar
          filters={props.filters}
          models={props.models}
          providers={props.providers}
          autoRefresh={props.autoRefresh}
          timingExpanded={props.timingExpanded}
          onChange={props.onFiltersChange}
          onAutoRefreshChange={props.onAutoRefreshChange}
          onTimingExpandedChange={props.onTimingExpandedChange}
        />
        <RequestRecordsTable
          rows={props.records.items}
          total={props.records.total}
          table={props.table}
          locale={props.locale}
          loading={props.records.isLoading}
          timingExpanded={props.timingExpanded}
          onOpen={props.onOpenRecord}
        />
      </Card>
      <RequestRecordDetailDrawer
        open={Boolean(props.displaySelectedRecord)}
        record={props.displaySelectedRecord}
        locale={props.locale}
        onClose={props.onCloseRecord}
      />
    </DashboardContent>
  );
}

function latestSelectedRecord(selectedRecord: RequestRecord | null, items: RequestRecord[]) {
  if (!selectedRecord) return null;
  return items.find((record) => record.request_id === selectedRecord.request_id) ?? selectedRecord;
}
