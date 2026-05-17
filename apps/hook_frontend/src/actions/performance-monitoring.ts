'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  PerformanceMonitoringRange,
  PerformanceMonitoringOverviewResponse,
  PerformanceMonitoringRealtimeResponse,
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
  >(url, fetcher, swrOptions);

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
