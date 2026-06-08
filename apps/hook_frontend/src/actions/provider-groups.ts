'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ProviderGroup,
  ProviderKeyGroup,
  ProviderGroupCreate,
  ProviderGroupUpdate,
  ProviderKeyGroupCreate,
  ProviderKeyGroupUpdate,
  ProviderGroupListResponse,
  ProviderKeyGroupListResponse,
} from 'src/types/provider-group';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type ProviderGroupFilters = {
  search?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useProviderGroups(page: number, pageSize: number, filters: ProviderGroupFilters = {}) {
  const key = [endpoints.adminProviders.groups, { params: queryParams(page, pageSize, filters) }] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ProviderGroupListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => groupListState(data, isLoading, error, isValidating, revalidate), [
    data,
    error,
    isLoading,
    isValidating,
    revalidate,
  ]);
}

export function useProviderKeyGroups(page: number, pageSize: number, filters: ProviderGroupFilters = {}) {
  const key = [endpoints.adminProviders.keyGroups, { params: queryParams(page, pageSize, filters) }] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ProviderKeyGroupListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => groupListState(data, isLoading, error, isValidating, revalidate), [
    data,
    error,
    isLoading,
    isValidating,
    revalidate,
  ]);
}

export async function createProviderGroup(payload: ProviderGroupCreate) {
  const group = await requestData<ProviderGroup>(axios.post(endpoints.adminProviders.groups, payload));
  await mutateProviderGroups();
  await mutateBillingGroups();
  return group;
}

export async function updateProviderGroup(id: string, payload: ProviderGroupUpdate) {
  const group = await requestData<ProviderGroup>(
    axios.patch(endpoints.adminProviders.groupById(id), payload)
  );
  await mutateProviderGroups();
  await mutateBillingGroups();
  return group;
}

export async function deleteProviderGroup(id: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.groupById(id)));
  await mutateProviderGroups();
  await mutateBillingGroups();
}

export async function createProviderKeyGroup(payload: ProviderKeyGroupCreate) {
  const group = await requestData<ProviderKeyGroup>(
    axios.post(endpoints.adminProviders.keyGroups, payload)
  );
  await mutateProviderKeyGroups();
  await mutateBillingGroups();
  return group;
}

export async function updateProviderKeyGroup(id: string, payload: ProviderKeyGroupUpdate) {
  const group = await requestData<ProviderKeyGroup>(
    axios.patch(endpoints.adminProviders.keyGroupById(id), payload)
  );
  await mutateProviderKeyGroups();
  await mutateBillingGroups();
  return group;
}

export async function deleteProviderKeyGroup(id: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.keyGroupById(id)));
  await mutateProviderKeyGroups();
  await mutateBillingGroups();
}

export async function mutateProviderGroups() {
  await mutate((key) => isProviderGroupKey(key));
}

export async function mutateProviderKeyGroups() {
  await mutate((key) => isProviderKeyGroupKey(key));
}

function groupListState<T>(
  data: ApiEnvelope<{ groups: T[]; total: number }> | undefined,
  isLoading: boolean,
  error: unknown,
  isValidating: boolean,
  refresh: () => Promise<ApiEnvelope<{ groups: T[]; total: number }> | undefined>
) {
  const pageData = data ? requireApiData(data) : undefined;
  return {
    data: pageData,
    items: pageData?.groups ?? [],
    total: pageData?.total ?? 0,
    isLoading,
    error,
    isValidating,
    refresh,
  };
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

function queryParams(page: number, pageSize: number, filters: ProviderGroupFilters) {
  return { skip: page * pageSize, limit: pageSize, ...filters };
}

function isProviderGroupKey(key: unknown) {
  return key === endpoints.adminProviders.groups || isEndpointArrayKey(key, endpoints.adminProviders.groups);
}

function isProviderKeyGroupKey(key: unknown) {
  return key === endpoints.adminProviders.keyGroups || isEndpointArrayKey(key, endpoints.adminProviders.keyGroups);
}

async function mutateBillingGroups() {
  await mutate((key) => key === endpoints.adminGroups.list || isEndpointArrayKey(key, endpoints.adminGroups.list));
}

function isEndpointArrayKey(key: unknown, endpoint: string) {
  return Array.isArray(key) && key[0] === endpoint;
}
