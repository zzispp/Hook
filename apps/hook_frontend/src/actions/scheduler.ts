'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ScheduledTask,
  ScheduledTaskUpdate,
  ScheduledTaskRunPage,
  ScheduledTaskRunFilters,
} from 'src/types/scheduler';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useScheduledTasks() {
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ScheduledTask[]>
  >(endpoints.adminScheduledTasks.list, fetcher, swrOptions);

  return useMemo(() => {
    const items = data ? requireApiData(data) : undefined;
    return {
      items: items ?? [],
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export function useScheduledTaskRuns(page: number, pageSize: number, filters: ScheduledTaskRunFilters = {}) {
  const disabled = page < 0 || pageSize <= 0;
  const key = disabled
    ? null
    : [
        endpoints.adminScheduledTasks.runs,
        {
          params: {
            ...pageQuery(page, pageSize),
            ...filters,
          },
        },
      ];
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ScheduledTaskRunPage>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading: disabled ? false : isLoading,
      error,
      isValidating: disabled ? false : isValidating,
      refresh: revalidate,
    };
  }, [data, disabled, error, isLoading, isValidating, revalidate]);
}

export async function updateScheduledTask(code: string, payload: ScheduledTaskUpdate) {
  const response = await axios.patch<ApiEnvelope<ScheduledTask>>(
    endpoints.adminScheduledTasks.byCode(code),
    payload
  );
  const task = requireApiData(response.data);
  await refreshScheduledTaskResources(code);
  return task;
}

async function refreshScheduledTaskResources(code: string) {
  await mutate(endpoints.adminScheduledTasks.list);
  await mutate((key) => isEndpointKey(key, endpoints.adminScheduledTasks.runs));
  await mutate(endpoints.adminScheduledTasks.byCode(code));
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}
