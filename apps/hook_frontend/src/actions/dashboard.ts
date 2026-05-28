'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  DashboardScope,
  DashboardPreset,
  DashboardUserStatsMetric,
  DashboardActivityResponse,
  DashboardOverviewResponse,
  DashboardFilterOptionsResponse,
  DashboardUserUsageStatsResponse,
  DashboardUserStatsTimeSeriesPoint,
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
  preset: DashboardPreset;
  userId?: string;
  compareUserId?: string;
  metric: DashboardUserStatsMetric;
  leaderboardPage: number;
  leaderboardPageSize: number;
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
        preset: filters.preset,
        metric: filters.metric,
        limit: filters.leaderboardPageSize,
        offset: filters.leaderboardPage * filters.leaderboardPageSize,
        tz_offset_minutes: timezoneOffsetMinutes(),
      })
    : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserStatsLeaderboardResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardUserUsageStats(enabled: boolean, filters: DashboardUserStatsFilters) {
  const params = compactUserStatsParams({
    preset: filters.preset,
    user_id: filters.userId,
    tz_offset_minutes: timezoneOffsetMinutes(),
  });
  const url = enabled ? dashboardUrl(endpoints.dashboard.userUsageStats, params) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<DashboardUserUsageStatsResponse>
  >(url, fetcher, swrOptions);

  return useDashboardData(data, isLoading, error, isValidating, mutate);
}

export function useDashboardUserStatsTimeSeries(enabled: boolean, filters: DashboardUserStatsFilters) {
  const params = compactUserStatsParams({
    preset: filters.preset,
    user_id: filters.userId,
    tz_offset_minutes: timezoneOffsetMinutes(),
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
    preset: filters.preset,
    user_id: filters.compareUserId,
    tz_offset_minutes: timezoneOffsetMinutes(),
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
