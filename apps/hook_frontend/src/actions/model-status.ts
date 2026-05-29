'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ModelStatusCheck,
  ModelStatusListFilters,
  ModelStatusCheckCreate,
  ModelStatusCheckUpdate,
  ModelStatusRunListFilters,
  ModelStatusRunListResponse,
  ModelStatusCheckBatchCreate,
  ModelStatusCheckListResponse,
  ModelStatusBatchUpdateRequest,
  ModelStatusBatchCreateResponse,
  ModelStatusBatchDeleteResponse,
  ModelStatusBatchUpdateResponse,
} from 'src/types/model-status';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useModelStatusChecks(filters: ModelStatusListFilters) {
  return useModelStatusList(endpoints.modelStatus.checks, filters);
}

export function useAdminModelStatusChecks(filters: ModelStatusListFilters) {
  return useModelStatusList(endpoints.adminModelStatus.checks, filters);
}

export function useAdminModelStatusRuns(page: number, pageSize: number, filters: ModelStatusRunListFilters) {
  const key = [
    endpoints.adminModelStatus.runs,
    { params: compactParams({ page, page_size: pageSize, ...filters }) },
  ] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ModelStatusRunListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.items ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function createModelStatusCheck(payload: ModelStatusCheckCreate) {
  const check = await requestData<ModelStatusCheck>(
    axios.post(endpoints.adminModelStatus.checks, payload)
  );
  await mutateModelStatus();
  return check;
}

export async function batchCreateModelStatusChecks(payload: ModelStatusCheckBatchCreate) {
  const response = await requestData<ModelStatusBatchCreateResponse>(
    axios.post(endpoints.adminModelStatus.batchCreate, payload)
  );
  await mutateModelStatus();
  return response;
}

export async function updateModelStatusCheck(id: string, payload: ModelStatusCheckUpdate) {
  const check = await requestData<ModelStatusCheck>(
    axios.patch(endpoints.adminModelStatus.byId(id), payload)
  );
  await mutateModelStatus();
  return check;
}

export async function batchDeleteModelStatusChecks(ids: string[]) {
  const response = await requestData<ModelStatusBatchDeleteResponse>(
    axios.post(endpoints.adminModelStatus.batchDelete, { ids })
  );
  await mutateModelStatus();
  return response;
}

export async function batchUpdateModelStatusChecks(payload: ModelStatusBatchUpdateRequest) {
  const response = await requestData<ModelStatusBatchUpdateResponse>(
    axios.post(endpoints.adminModelStatus.batchUpdate, payload)
  );
  await mutateModelStatus();
  return response;
}

function useModelStatusList(endpoint: string, filters: ModelStatusListFilters) {
  const key = [endpoint, { params: compactParams(filters) }] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ModelStatusCheckListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.checks ?? [],
      total: pageData?.checks.length ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function mutateModelStatus() {
  await mutate((key) => isModelStatusKey(key, endpoints.modelStatus.checks));
  await mutate((key) => isModelStatusKey(key, endpoints.adminModelStatus.checks));
  await mutate((key) => isModelStatusKey(key, endpoints.adminModelStatus.runs));
}

function isModelStatusKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function compactParams(filters: Record<string, string | number | boolean | undefined>) {
  return Object.fromEntries(
    Object.entries(filters).filter((entry): entry is [string, string | number | boolean] => {
      const value = entry[1];
      return value !== undefined && value !== '';
    })
  );
}
