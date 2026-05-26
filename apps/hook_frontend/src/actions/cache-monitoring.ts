'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  CacheAffinityIdentity,
  CacheAffinityPageResponse,
} from 'src/types/cache-monitoring';

import useSWR, { mutate } from 'swr';
import { useMemo, useCallback } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useCacheAffinities(page: number, pageSize: number, search: string) {
  const key = [
    endpoints.cacheMonitoring.affinities,
    { params: { ...pageQuery(page, pageSize), search: search || undefined } },
  ] as const;
  const { data, isLoading, error, isValidating, mutate: refreshData } = useSWR<
    ApiEnvelope<CacheAffinityPageResponse>
  >(key, fetcher, swrOptions);
  const refresh = useCallback(() => refreshData(), [refreshData]);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh,
    };
  }, [data, error, isLoading, isValidating, refresh]);
}

export async function deleteCacheAffinity(item: CacheAffinityIdentity) {
  await requestData<void>(
    axios.delete(
      endpoints.cacheMonitoring.affinityById(
        item.affinity_key,
        item.endpoint_id,
        item.model_id,
        item.api_format
      )
    )
  );
  await mutate(isCacheMonitoringKey);
}

export async function clearCacheAffinities() {
  await requestSuccess(axios.delete(endpoints.cacheMonitoring.clearAll));
  await mutate(isCacheMonitoringKey);
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function requestSuccess(request: Promise<{ data: ApiEnvelope<unknown> }>) {
  const response = await request;
  if (!response.data.success) {
    throw new Error(response.data.message || 'Request failed');
  }
}

function isCacheMonitoringKey(key: unknown) {
  return Array.isArray(key) && key[0] === endpoints.cacheMonitoring.affinities;
}
