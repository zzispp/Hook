'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  Provider,
  ProviderApiKey,
  ProviderCreate,
  ProviderUpdate,
  ProviderCooldown,
  ProviderEndpoint,
  ProviderApiKeyCreate,
  ProviderApiKeyUpdate,
  ProviderListResponse,
  ProviderModelBinding,
  ProviderEndpointCreate,
  ProviderEndpointUpdate,
  ProviderModelTestRequest,
  ProviderModelTestResponse,
  ProviderModelBindingCreate,
  ProviderModelBindingUpdate,
  ProviderModelCostBatchUpsert,
  ProviderCooldownListResponse,
  ProviderModelCostListResponse,
  ProviderUpstreamModelsResponse,
  ProviderApiKeyPriorityBatchUpdate,
} from 'src/types/provider';

import useSWR, { mutate } from 'swr';
import { useMemo, useCallback } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type ProviderFilters = {
  api_format?: string;
  is_active?: boolean;
  model_id?: string;
  search?: string;
};

export type ProviderCooldownFilters = {
  search?: string;
  status_code?: number;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useProviders(page: number, pageSize: number, filters: ProviderFilters = {}) {
  const key = [
    endpoints.adminProviders.list,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;

  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<ApiEnvelope<ProviderListResponse>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.providers ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export function useProviderEndpoints(providerId?: string | null) {
  return useProviderChildResource<ProviderEndpoint>(providerId, endpoints.adminProviders.endpoints);
}

export function useProviderApiKeys(providerId?: string | null) {
  return useProviderChildResource<ProviderApiKey>(providerId, endpoints.adminProviders.keys);
}

export function useProviderPriorityKeys(providers: Provider[]) {
  const providerIds = useMemo(() => providers.map((provider) => provider.id), [providers]);
  const key = providerIds.length > 0 ? providerPriorityKeysCacheKey(providerIds) : null;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    Record<string, ProviderApiKey[]>
  >(key, () => fetchProviderKeysByProvider(providerIds), swrOptions);
  const refreshForProviders = useCallback(async (nextProviders: Pick<Provider, 'id'>[]) => {
    const nextProviderIds = nextProviders.map((provider) => provider.id);
    if (nextProviderIds.length === 0) return {};
    const nextData = await fetchProviderKeysByProvider(nextProviderIds);
    await mutate(providerPriorityKeysCacheKey(nextProviderIds), nextData, false);
    return nextData;
  }, []);

  return useMemo(
    () => ({
      itemsByProvider: data ?? {},
      isLoading: providerIds.length > 0 ? isLoading : false,
      error,
      isValidating: providerIds.length > 0 ? isValidating : false,
      refresh: revalidate,
      refreshForProviders,
    }),
    [data, error, isLoading, isValidating, providerIds.length, refreshForProviders, revalidate]
  );
}

export function useProviderKeysByProvider(providers: Pick<Provider, 'id'>[]) {
  const providerIds = useMemo(() => providers.map((provider) => provider.id), [providers]);
  const key = providerIds.length > 0 ? ['provider-keys-by-provider', providerIds] : null;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    Record<string, ProviderApiKey[]>
  >(key, () => fetchProviderKeysByProvider(providerIds), swrOptions);

  return useMemo(
    () => ({
      itemsByProvider: data ?? {},
      isLoading: providerIds.length > 0 ? isLoading : false,
      error,
      isValidating: providerIds.length > 0 ? isValidating : false,
      refresh: revalidate,
    }),
    [data, error, isLoading, isValidating, providerIds.length, revalidate]
  );
}

export function useProviderModels(providerId?: string | null) {
  return useProviderChildResource<ProviderModelBinding>(providerId, endpoints.adminProviders.models);
}

export function useProviderModelCosts(providerId?: string | null) {
  const key = providerId ? endpoints.adminProviders.modelCosts(providerId) : null;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<ProviderModelCostListResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      items: pageData?.costs ?? [],
      isLoading: providerId ? isLoading : false,
      error,
      isValidating: providerId ? isValidating : false,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, providerId, revalidate]);
}

export function useProviderCooldowns(
  page: number,
  pageSize: number,
  filters: ProviderCooldownFilters = {}
) {
  const key = [
    endpoints.adminProviders.cooldowns,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;
  const {
    data,
    isLoading,
    error,
    isValidating,
    mutate: revalidate,
  } = useSWR<ApiEnvelope<ProviderCooldownListResponse>>(key, fetcher, swrOptions);

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.cooldowns ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function createProvider(payload: ProviderCreate) {
  const provider = await requestData<Provider>(axios.post(endpoints.adminProviders.list, payload));
  await mutateProviders();
  await mutateProviderGroups();
  return provider;
}

export async function updateProvider(id: string, payload: ProviderUpdate) {
  const provider = await requestData<Provider>(axios.patch(endpoints.adminProviders.byId(id), payload));
  await mutateProviders();
  return provider;
}

export async function deleteProvider(id: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.byId(id)));
  await mutateProviders();
  await mutateProviderGroups();
  await mutateProviderKeyGroups();
  await mutateProviderCooldowns();
}

export async function releaseProviderCooldown(providerId: string) {
  const cooldown = await requestData<ProviderCooldown>(
    axios.post(endpoints.adminProviders.releaseCooldown(providerId))
  );
  await mutateProviderCooldowns();
  return cooldown;
}

export async function createProviderEndpoint(providerId: string, payload: ProviderEndpointCreate) {
  const endpoint = await requestData<ProviderEndpoint>(
    axios.post(endpoints.adminProviders.endpoints(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  return endpoint;
}

export async function updateProviderEndpoint(
  providerId: string,
  endpointId: string,
  payload: ProviderEndpointUpdate
) {
  const endpoint = await requestData<ProviderEndpoint>(
    axios.patch(endpoints.adminProviders.endpointById(providerId, endpointId), payload)
  );
  await mutateProviderChildren(providerId);
  return endpoint;
}

export async function deleteProviderEndpoint(providerId: string, endpointId: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.endpointById(providerId, endpointId)));
  await mutateProviderChildren(providerId);
}

export async function createProviderApiKey(providerId: string, payload: ProviderApiKeyCreate) {
  const apiKey = await requestData<ProviderApiKey>(
    axios.post(endpoints.adminProviders.keys(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  return apiKey;
}

export async function updateProviderApiKey(
  providerId: string,
  keyId: string,
  payload: ProviderApiKeyUpdate
) {
  const apiKey = await requestData<ProviderApiKey>(
    axios.patch(endpoints.adminProviders.keyById(providerId, keyId), payload)
  );
  await mutateProviderChildren(providerId);
  return apiKey;
}

export async function updateProviderApiKeyPriorities(payload: ProviderApiKeyPriorityBatchUpdate) {
  const apiKeys = await requestData<ProviderApiKey[]>(
    axios.post(endpoints.adminProviders.keyBatchPriorities, payload)
  );
  await mutateProviders();
  await Promise.all([...new Set(payload.updates.map((update) => update.provider_id))].map(mutateProviderChildren));
  return apiKeys;
}

export async function fetchProviderUpstreamModels(providerId: string) {
  const response = await requestData<ProviderUpstreamModelsResponse>(
    axios.get(endpoints.adminProviders.upstreamModels(providerId))
  );
  return response.models;
}

export async function deleteProviderApiKey(providerId: string, keyId: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.keyById(providerId, keyId)));
  await mutateProviderChildren(providerId);
  await mutateProviderKeyGroups();
}

export async function createProviderModel(providerId: string, payload: ProviderModelBindingCreate) {
  const model = await requestData<ProviderModelBinding>(
    axios.post(endpoints.adminProviders.models(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  return model;
}

export async function updateProviderModel(
  providerId: string,
  modelId: string,
  payload: ProviderModelBindingUpdate
) {
  const model = await requestData<ProviderModelBinding>(
    axios.patch(endpoints.adminProviders.modelById(providerId, modelId), payload)
  );
  await mutateProviderChildren(providerId);
  return model;
}

export async function deleteProviderModel(providerId: string, modelId: string) {
  await requestSuccess(axios.delete(endpoints.adminProviders.modelById(providerId, modelId)));
  await mutateProviderChildren(providerId);
}

export async function upsertProviderModelCosts(
  providerId: string,
  keyId: string,
  payload: ProviderModelCostBatchUpsert
) {
  const response = await requestData<ProviderModelCostListResponse>(
    axios.put(endpoints.adminProviders.keyModelCosts(providerId, keyId), payload)
  );
  await mutateProviderChildren(providerId);
  return response.costs;
}

export async function deleteProviderModelCost(
  providerId: string,
  keyId: string,
  providerModelId: string
) {
  await requestSuccess(
    axios.delete(endpoints.adminProviders.keyModelCostByModel(providerId, keyId, providerModelId))
  );
  await mutateProviderChildren(providerId);
}

export async function testProviderModel(
  providerId: string,
  modelId: string,
  payload: ProviderModelTestRequest
) {
  return requestData<ProviderModelTestResponse>(
    axios.post(endpoints.adminProviders.modelTest(providerId, modelId), payload)
  );
}

function useProviderChildResource<T>(
  providerId: string | null | undefined,
  endpoint: (id: string) => string
) {
  const key = providerId ? endpoint(providerId) : null;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<ApiEnvelope<T[]>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const items = data ? requireApiData(data) : undefined;
    return {
      items: items ?? [],
      isLoading: providerId ? isLoading : false,
      error,
      isValidating: providerId ? isValidating : false,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, providerId, revalidate]);
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

async function fetchProviderKeysByProvider(providerIds: string[]) {
  const pairs = await Promise.all(
    providerIds.map(async (providerId) => {
      const keys = await requestData<ProviderApiKey[]>(axios.get(endpoints.adminProviders.keys(providerId)));
      return [providerId, keys] as const;
    })
  );
  return Object.fromEntries(pairs);
}

function providerPriorityKeysCacheKey(providerIds: string[]) {
  return ['provider-priority-keys', providerIds] as const;
}

async function mutateProviders() {
  await mutate((key) => isProviderKey(key));
}

async function mutateProviderCooldowns() {
  await mutate((key) => isEndpointKey(key, endpoints.adminProviders.cooldowns));
}

async function mutateProviderGroups() {
  await mutate((key) => isEndpointKey(key, endpoints.adminProviders.groups));
}

async function mutateProviderKeyGroups() {
  await mutate((key) => isEndpointKey(key, endpoints.adminProviders.keyGroups));
}

async function mutateProviderChildren(providerId: string) {
  await mutate(
    (key) =>
      (typeof key === 'string' && key.startsWith(`/api/admin/providers/${providerId}/`)) ||
      isProviderPriorityKeysKey(key, providerId)
  );
}

function isProviderKey(key: unknown) {
  return key === endpoints.adminProviders.list || (Array.isArray(key) && key[0] === endpoints.adminProviders.list);
}

function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

function isProviderPriorityKeysKey(key: unknown, providerId: string) {
  return (
    Array.isArray(key) &&
    key[0] === 'provider-priority-keys' &&
    Array.isArray(key[1]) &&
    key[1].includes(providerId)
  );
}
