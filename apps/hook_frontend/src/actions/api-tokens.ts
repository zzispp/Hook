'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ApiToken,
  ApiTokenCreate,
  ApiTokenUpdate,
  AdminApiTokenCreate,
  ApiTokenListResponse,
  ApiTokenCreateResponse,
  ApiTokenSecretResponse,
} from 'src/types/api-token';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type ApiTokenFilters = {
  is_active?: boolean;
  search?: string;
  token_type?: string;
  user_id?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useApiTokens(page: number, pageSize: number, filters: ApiTokenFilters = {}) {
  return useTokenList(endpoints.apiTokens.list, page, pageSize, filters);
}

export function useAdminApiTokens(page: number, pageSize: number, filters: ApiTokenFilters = {}) {
  return useTokenList(endpoints.adminApiTokens.list, page, pageSize, filters);
}

export function useScopedApiTokens(scope: 'user' | 'admin', page: number, pageSize: number, filters: ApiTokenFilters = {}) {
  const endpoint = scope === 'admin' ? endpoints.adminApiTokens.list : endpoints.apiTokens.list;
  return useTokenList(endpoint, page, pageSize, filters);
}

function useTokenList(endpoint: string, page: number, pageSize: number, filters: ApiTokenFilters) {
  const disabled = page < 0 || pageSize <= 0;
  const key = [endpoint, { params: { skip: page * pageSize, limit: pageSize, ...filters } }] as const;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ApiTokenListResponse>
  >(disabled ? null : key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.tokens ?? [],
      total: pageData?.total ?? 0,
      isLoading: disabled ? false : isLoading,
      error,
      isValidating: disabled ? false : isValidating,
      refresh: revalidate,
    };
  }, [data, disabled, error, isLoading, isValidating, revalidate]);
}

export async function createApiToken(payload: ApiTokenCreate) {
  const token = await requestData<ApiTokenCreateResponse>(axios.post(endpoints.apiTokens.list, payload));
  await mutateApiTokens();
  return token;
}

export async function createAdminApiToken(payload: AdminApiTokenCreate) {
  const token = await requestData<ApiTokenCreateResponse>(
    axios.post(endpoints.adminApiTokens.list, payload)
  );
  await mutateApiTokens();
  return token;
}

export async function updateApiToken(id: string, payload: ApiTokenUpdate) {
  const token = await requestData<ApiToken>(axios.patch(endpoints.apiTokens.byId(id), payload));
  await mutateApiTokens();
  return token;
}

export async function updateAdminApiToken(id: string, payload: ApiTokenUpdate) {
  const token = await requestData<ApiToken>(axios.patch(endpoints.adminApiTokens.byId(id), payload));
  await mutateApiTokens();
  return token;
}

export async function deleteApiToken(id: string) {
  await requestSuccess(axios.delete(endpoints.apiTokens.byId(id)));
  await mutateApiTokens();
}

export async function deleteAdminApiToken(id: string) {
  await requestSuccess(axios.delete(endpoints.adminApiTokens.byId(id)));
  await mutateApiTokens();
}

export async function getApiTokenSecret(id: string) {
  return requestData<ApiTokenSecretResponse>(axios.get(endpoints.apiTokens.secret(id)));
}

export async function getAdminApiTokenSecret(id: string) {
  return requestData<ApiTokenSecretResponse>(axios.get(endpoints.adminApiTokens.secret(id)));
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

async function mutateApiTokens() {
  await mutate((key) => isApiTokenKey(key, endpoints.apiTokens.list));
  await mutate((key) => isApiTokenKey(key, endpoints.adminApiTokens.list));
}

function isApiTokenKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}
