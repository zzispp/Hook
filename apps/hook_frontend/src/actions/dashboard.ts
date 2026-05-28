'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  DashboardScope,
  DashboardPreset,
  DashboardSortOrder,
  DashboardUserStatsMetric,
  DashboardActivityResponse,
  DashboardOverviewResponse,
  DashboardCostAnalysisPreset,
  DashboardCostSavingsResponse,
  DashboardCostForecastResponse,
  DashboardFilterOptionsResponse,
  DashboardUserUsageStatsResponse,
  DashboardProviderAggregationItem,
  DashboardUserStatsTimeSeriesPoint,
  DashboardApiKeyLeaderboardResponse,
  DashboardUserStatsLeaderboardResponse,
} from 'src/types/dashboard';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useDashboardOverview(
  preset: DashboardPreset,
  filters: DashboardScopeFilters,
  page: number,
  pageSize: number
) {
  const url = dashboardRequestReady(filters)
    ? dashboardUrl(endpoints.dashboard.overview, {
        ...compactParams(filters),
        ...pageQuery(page, pageSize),
        preset,
        tz_offset_minutes: timezoneOffsetMinutes(),
      })
    : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardOverviewResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export type DashboardScopeFilters = {
  scope: DashboardScope['scope'];
  user_id?: string;
  token_id?: string;
};

export type DashboardActivityFilters = DashboardScopeFilters;

export type DashboardUserStatsFilters = {
  preset?: DashboardPreset;
  start_date?: string;
  end_date?: string;
  userId?: string;
  compareUserId?: string;
  metric: DashboardUserStatsMetric;
  leaderboardPage: number;
  leaderboardPageSize: number;
};

export type DashboardCostAnalysisFilters = {
  preset: DashboardCostAnalysisPreset;
  start_date?: string;
  end_date?: string;
};

export type DashboardApiKeyLeaderboardFilters = DashboardCostAnalysisFilters & {
  metric: DashboardUserStatsMetric;
  order: DashboardSortOrder;
  page: number;
  pageSize: number;
};

export function useDashboardActivity(filters: DashboardActivityFilters) {
  const url = dashboardRequestReady(filters)
    ? dashboardUrl(endpoints.dashboard.activity, {
        ...compactParams(filters),
        tz_offset_minutes: timezoneOffsetMinutes(),
      })
    : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardActivityResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardFilterOptions(enabled: boolean) {
  const url = enabled
    ? dashboardUrl(endpoints.dashboard.filterOptions, {
        tz_offset_minutes: timezoneOffsetMinutes(),
      })
    : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardFilterOptionsResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardUserStatsLeaderboard(enabled: boolean, filters: DashboardUserStatsFilters) {
  const url = enabled
    ? dashboardUrl(endpoints.dashboard.userStatsLeaderboard, {
        ...userStatsRangeParams(filters),
        metric: filters.metric,
        limit: filters.leaderboardPageSize,
        offset: filters.leaderboardPage * filters.leaderboardPageSize,
      })
    : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserStatsLeaderboardResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardUserUsageStats(enabled: boolean, filters: DashboardUserStatsFilters) {
  const params = compactUserStatsParams({
    ...userStatsRangeParams(filters),
    user_id: filters.userId,
  });
  const url = enabled ? dashboardUrl(endpoints.dashboard.userUsageStats, params) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserUsageStatsResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardUserStatsTimeSeries(enabled: boolean, filters: DashboardUserStatsFilters) {
  const params = compactUserStatsParams({
    ...userStatsRangeParams(filters),
    user_id: filters.userId,
  });
  const url = enabled ? dashboardUrl(endpoints.dashboard.userStatsTimeSeries, params) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserStatsTimeSeriesPoint[]>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardCompareUserStatsTimeSeries(
  enabled: boolean,
  filters: DashboardUserStatsFilters
) {
  const params = compactUserStatsParams({
    ...userStatsRangeParams(filters),
    user_id: filters.compareUserId,
  });
  const url =
    enabled && filters.compareUserId
      ? dashboardUrl(endpoints.dashboard.userStatsTimeSeries, params)
      : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserStatsTimeSeriesPoint[]>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardCostForecast(filters: DashboardCostAnalysisFilters) {
  const url = dashboardUrl(endpoints.dashboard.costForecast, costAnalysisParams(filters));
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardCostForecastResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardCostSavings(filters: DashboardCostAnalysisFilters) {
  const url = dashboardUrl(endpoints.dashboard.costSavings, costAnalysisParams(filters));
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardCostSavingsResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardApiKeyLeaderboard(filters: DashboardApiKeyLeaderboardFilters) {
  const url = dashboardUrl(endpoints.dashboard.apiKeyLeaderboard, {
    ...costAnalysisParams(filters),
    metric: filters.metric,
    order: filters.order,
    limit: filters.pageSize,
    offset: filters.page * filters.pageSize,
    include_inactive: 'false',
    exclude_admin: 'false',
  });
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardApiKeyLeaderboardResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardProviderAggregation(filters: DashboardCostAnalysisFilters) {
  const url = dashboardUrl(endpoints.dashboard.providerAggregation, {
    ...costAnalysisParams(filters),
    group_by: 'provider',
    limit: 8,
  });
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardProviderAggregationItem[]>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

function useDashboardData<T>(
  data: ApiEnvelope<T> | undefined,
  isLoading: boolean,
  error: unknown,
  isValidating: boolean,
  mutate: VoidFunction
) {
  return useMemo(() => {
    const apiError =
      data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    return {
      data: data?.success ? requireApiData(data) : undefined,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

function dashboardUrl(endpoint: string, params: Record<string, string | number>) {
  return `${endpoint}?${new URLSearchParams(stringParams(params)).toString()}`;
}

function compactParams(params: DashboardScopeFilters): Record<string, string> {
  return Object.fromEntries(
    Object.entries(params).filter((entry): entry is [string, string] => Boolean(entry[1]))
  );
}

function compactUserStatsParams(params: Record<string, string | number | undefined>) {
  return Object.fromEntries(
    Object.entries(params).filter((entry): entry is [string, string | number] => {
      const value = entry[1];
      return value !== undefined && value !== '';
    })
  );
}

function costAnalysisParams(filters: DashboardCostAnalysisFilters) {
  return compactUserStatsParams({
    preset: filters.preset,
    start_date: filters.start_date,
    end_date: filters.end_date,
    tz_offset_minutes: timezoneOffsetMinutes(),
  });
}

function userStatsRangeParams(filters: DashboardUserStatsFilters) {
  return compactUserStatsParams({
    preset: filters.preset,
    start_date: filters.start_date,
    end_date: filters.end_date,
    tz_offset_minutes: timezoneOffsetMinutes(),
  });
}

function dashboardRequestReady(filters: DashboardScopeFilters) {
  if (filters.scope === 'user') return Boolean(filters.user_id);
  if (filters.scope === 'token') return Boolean(filters.token_id);
  return true;
}

function stringParams(params: Record<string, string | number>) {
  return Object.fromEntries(Object.entries(params).map(([key, value]) => [key, String(value)]));
}

function timezoneOffsetMinutes() {
  return -new Date().getTimezoneOffset();
}
