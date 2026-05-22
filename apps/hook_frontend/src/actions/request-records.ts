'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  RequestRecord,
  RequestRecordDetail,
  UsageRecordListResponse,
  RequestRecordListResponse,
  ActiveRequestRecordResponse,
} from 'src/types/provider';

import useSWR, { useSWRConfig } from 'swr';
import { useMemo, useCallback } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type RequestRecordFilters = {
  api_format?: string;
  model_id?: string;
  provider_id?: string;
  search?: string;
  status?: string;
  type?: string;
};

const listSwrOptions = {
  keepPreviousData: false,
  revalidateOnFocus: false,
};

const detailSwrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useRequestRecords(
  page: number,
  pageSize: number,
  filters: RequestRecordFilters = {}
) {
  const { mutate: globalMutate } = useSWRConfig();
  const key = useMemo(
    () =>
      [
        endpoints.adminRequestRecords.list,
        { params: { skip: page * pageSize, limit: pageSize, ...filters } },
      ] as const,
    [filters, page, pageSize]
  );
  const { data, isLoading, error, isValidating } = useSWR<
    ApiEnvelope<RequestRecordListResponse>
  >(key, fetcher, listSwrOptions);
  const refresh = useCallback(() => globalMutate(key), [globalMutate, key]);
  const updateItems = useCallback(
    (updater: (items: RequestRecord[]) => RequestRecord[]) => {
      void globalMutate(
        key,
        (current) => {
          if (!current?.data) return current;
          return {
            ...current,
            data: {
              ...current.data,
              records: updater(current.data.records),
            },
          };
        },
        { revalidate: false }
      );
    },
    [globalMutate, key]
  );

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.records ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh,
      updateItems,
    };
  }, [data, error, isLoading, isValidating, refresh, updateItems]);
}

export function useUsageRecords(
  page: number,
  pageSize: number,
  filters: Omit<RequestRecordFilters, 'provider_id'> = {}
) {
  const key = [
    endpoints.usageRecords.list,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<UsageRecordListResponse>
  >(key, fetcher, listSwrOptions);
  const refresh = useCallback(() => mutate(), [mutate]);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.records ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh,
    };
  }, [data, error, isLoading, isValidating, refresh]);
}

export async function fetchActiveRequestRecords(ids: string[], signal?: AbortSignal) {
  const response = await axios.post<ApiEnvelope<ActiveRequestRecordResponse>>(
    endpoints.adminRequestRecords.active,
    { ids },
    { signal }
  );
  return requireApiData(response.data);
}

export function useRequestRecordDetail(requestId?: string | null) {
  const key = requestId ? endpoints.adminRequestRecords.byId(requestId) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<RequestRecordDetail>>(
    key,
    fetcher,
    detailSwrOptions
  );
  const refresh = useCallback(() => mutate(), [mutate]);

  return useMemo(() => {
    const detail = data ? requireApiData(data) : undefined;
    return {
      data: detail,
      isLoading: requestId ? isLoading : false,
      error,
      isValidating: requestId ? isValidating : false,
      refresh,
    };
  }, [data, error, isLoading, isValidating, refresh, requestId]);
}
