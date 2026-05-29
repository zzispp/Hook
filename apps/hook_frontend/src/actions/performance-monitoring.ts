'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  PerformanceMonitoringRange,
  PerformanceMonitoringAnalyticsQuery,
  PerformanceMonitoringOverviewResponse,
  PerformanceMonitoringRealtimeResponse,
  PerformanceMonitoringAnalyticsResponse,
} from 'src/types/performance-monitoring';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function usePerformanceMonitoringOverview(range: PerformanceMonitoringRange | null) {
  const url = range ? `${endpoints.performanceMonitoring.overview}?range=${range}` : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<PerformanceMonitoringOverviewResponse>
  >(url, fetcher, {
    ...swrOptions,
    refreshInterval: range === 'realtime' ? 5000 : 0,
  });

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

export function usePerformanceMonitoringRealtime(enabled = true) {
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<PerformanceMonitoringRealtimeResponse>
  >(enabled ? endpoints.performanceMonitoring.realtime : null, fetcher, {
    ...swrOptions,
    refreshInterval: 5000,
  });

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

export function usePerformanceMonitoringAnalytics(query: PerformanceMonitoringAnalyticsQuery | null) {
  const url = query ? analyticsUrl(query) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<PerformanceMonitoringAnalyticsResponse>
  >(url, fetcher, {
    ...swrOptions,
    refreshInterval: query?.range === 'realtime' ? 5000 : 0,
  });

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

function analyticsUrl(query: PerformanceMonitoringAnalyticsQuery) {
  const params = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (value === undefined || value === null || value === '') return;
    params.set(key, String(value));
  });
  return `${endpoints.performanceMonitoring.analytics}?${params.toString()}`;
}
