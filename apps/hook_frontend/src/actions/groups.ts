'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  BillingGroup,
  BillingGroupCreate,
  BillingGroupUpdate,
  BillingGroupListResponse,
} from 'src/types/group';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type BillingGroupFilters = {
  is_active?: boolean;
  search?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useBillingGroups(page: number, pageSize: number, filters: BillingGroupFilters = {}) {
  const key = [
    endpoints.adminGroups.list,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<BillingGroupListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.groups ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export function useAvailableBillingGroups(enabled = true) {
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<BillingGroup[]>
  >(enabled ? endpoints.groups.available : null, fetcher, swrOptions);

  return useMemo(() => {
    const groups = data ? requireApiData(data) : undefined;
    return {
      items: groups ?? [],
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function createBillingGroup(payload: BillingGroupCreate) {
  const group = await requestData<BillingGroup>(axios.post(endpoints.adminGroups.list, payload));
  await mutateBillingGroups();
  return group;
}

export async function updateBillingGroup(id: string, payload: BillingGroupUpdate) {
  const group = await requestData<BillingGroup>(axios.patch(endpoints.adminGroups.byId(id), payload));
  await mutateBillingGroups();
  return group;
}

export async function deleteBillingGroup(id: string) {
  await requestSuccess(axios.delete(endpoints.adminGroups.byId(id)));
  await mutateBillingGroups();
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

async function mutateBillingGroups() {
  await mutate((key) => key === endpoints.groups.available || isGroupKey(key));
}

function isGroupKey(key: unknown) {
  return Array.isArray(key) && key[0] === endpoints.adminGroups.list;
}
