'use client';

import { useRef, useState, useCallback } from 'react';

import Card from '@mui/material/Card';

import { useTranslate } from 'src/locales/use-locales';
import { useUserModelCatalog } from 'src/actions/models';
import { DashboardContent } from 'src/layouts/dashboard';
import { useUsageRecords } from 'src/actions/request-records';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { useTable } from 'src/components/table';

import { RefreshButton, AdminBreadcrumbs } from 'src/sections/admin/shared';
import { useAutoRefresh, usePageVisible } from 'src/sections/admin/request-records-polling';
import { DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE } from 'src/sections/admin/request-records-utils';

import { UsageRecordsTable } from './usage-records-table';
import {
  UsageRecordsToolbar,
  toUsageRecordQueryFilters,
  catalogUsageRecordOptions,
  type UsageRecordFilterState,
  DEFAULT_USAGE_RECORD_FILTERS,
} from './usage-records-toolbar';

const AUTO_REFRESH_INTERVAL_MS = 3000;

export function UsageRecordsView() {
  return <UsageRecordsContent {...useUsageRecordsViewProps()} />;
}

function useUsageRecordsViewProps() {
  const { currentLang } = useTranslate('admin');
  const table = useTable({
    defaultRowsPerPage: DEFAULT_REQUEST_RECORD_ROWS_PER_PAGE,
    defaultOrderBy: 'created_at',
  });
  const { filters, handleFiltersChange } = useUsageRecordFilters(table.onResetPage);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const models = useUserModelCatalog();
  const records = useUsageRecords(table.page, table.rowsPerPage, toUsageRecordQueryFilters(filters));
  const locale = currentLang.numberFormat.code;
  const pageVisible = usePageVisible();
  const refresh = useUsageRecordRefresh(records.refresh);

  useAutoRefresh(autoRefresh && pageVisible, refresh.backgroundRefresh, AUTO_REFRESH_INTERVAL_MS);

  return {
    table,
    filters,
    models: catalogUsageRecordOptions(models.items),
    records,
    autoRefresh,
    manualRefreshing: refresh.manualRefreshing,
    locale,
    onFiltersChange: handleFiltersChange,
    onAutoRefreshChange: setAutoRefresh,
    onManualRefresh: refresh.handleManualRefresh,
  };
}

function useUsageRecordFilters(resetPage: () => void) {
  const [filters, setFilters] = useState(DEFAULT_USAGE_RECORD_FILTERS);
  const handleFiltersChange = useCallback(
    (nextFilters: UsageRecordFilterState) => {
      resetPage();
      setFilters(nextFilters);
    },
    [resetPage]
  );
  return { filters, handleFiltersChange };
}

function useUsageRecordRefresh(refreshRecords: () => Promise<unknown> | unknown) {
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

type UsageRecordsContentProps = ReturnType<typeof useUsageRecordsViewProps>;

function UsageRecordsContent(props: UsageRecordsContentProps) {
  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.usageRecords}
        action={
          <RefreshButton
            loading={props.manualRefreshing}
            onClick={() => void props.onManualRefresh()}
          />
        }
      />
      <Card>
        <UsageRecordsToolbar
          filters={props.filters}
          models={props.models}
          autoRefresh={props.autoRefresh}
          onChange={props.onFiltersChange}
          onAutoRefreshChange={props.onAutoRefreshChange}
        />
        <UsageRecordsTable
          rows={props.records.items}
          total={props.records.total}
          table={props.table}
          locale={props.locale}
          loading={props.records.isLoading}
        />
      </Card>
    </DashboardContent>
  );
}
