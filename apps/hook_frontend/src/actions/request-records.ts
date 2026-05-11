'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  RequestRecord,
  RequestRecordDetail,
  RequestRecordListResponse,
  ActiveRequestRecordResponse,
} from 'src/types/provider';

import useSWR from 'swr';
import { useMemo, useCallback } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type RequestRecordFilters = {
  search?: string;
  status?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useRequestRecords(page: number, pageSize: number, filters: RequestRecordFilters = {}) {
  const key = [
    endpoints.adminRequestRecords.list,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<RequestRecordListResponse>
  >(key, fetcher, swrOptions);
  const refresh = useCallback(() => {
    void mutate();
  }, [mutate]);
  const updateItems = useCallback(
    (updater: (items: RequestRecord[]) => RequestRecord[]) => {
      void mutate(
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
    [mutate]
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

export async function fetchActiveRequestRecords(ids: string[]) {
  const response = await axios.post<ApiEnvelope<ActiveRequestRecordResponse>>(
    endpoints.adminRequestRecords.active,
    { ids }
  );
  return requireApiData(response.data);
}

export function useRequestRecordDetail(requestId?: string | null) {
  const key = requestId ? endpoints.adminRequestRecords.byId(requestId) : null;
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<RequestRecordDetail>>(
    key,
    fetcher,
    swrOptions
  );
  const refresh = useCallback(() => {
    void mutate();
  }, [mutate]);

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
