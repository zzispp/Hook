'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ModelsDevData,
  GlobalModelCreate,
  GlobalModelUpdate,
  ModelsDevModelItem,
  GlobalModelResponse,
  ModelCatalogResponse,
  GlobalModelListResponse,
  GlobalModelProvidersResponse,
  BatchDeleteGlobalModelsResponse,
} from 'src/types/model';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import {
  aetherCacheReadPrice,
  aetherCacheCreationPrice,
  aetherCache1hCreationPrice,
} from 'src/utils/model-pricing';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

// ----------------------------------------------------------------------

export type GlobalModelFilters = {
  is_active?: boolean;
  search?: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useGlobalModels(page: number, pageSize: number, filters: GlobalModelFilters = {}) {
  const key = [
    endpoints.adminModels.global,
    { params: { skip: page * pageSize, limit: pageSize, ...filters } },
  ] as const;

  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<ApiEnvelope<GlobalModelListResponse>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const pageData = data ? requireApiData(data) : undefined;
    return {
      data: pageData,
      items: pageData?.models ?? [],
      total: pageData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function createGlobalModel(payload: GlobalModelCreate) {
  const model = await requestData<GlobalModelResponse>(
    axios.post(endpoints.adminModels.global, payload)
  );
  await mutateGlobalModels();
  return model;
}

export async function updateGlobalModel(id: string, payload: GlobalModelUpdate) {
  const model = await requestData<GlobalModelResponse>(
    axios.patch(endpoints.adminModels.globalById(id), payload)
  );
  await mutateGlobalModels();
  return model;
}

export function useGlobalModelProviders(id: string | null | undefined) {
  const key = id ? endpoints.adminModels.globalProviders(id) : null;
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<GlobalModelProvidersResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const providerData = data ? requireApiData(data) : undefined;
    return {
      data: providerData,
      items: providerData?.providers ?? [],
      total: providerData?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function deleteGlobalModel(id: string) {
  await requestSuccess(axios.delete(endpoints.adminModels.globalById(id)));
  await mutateGlobalModels();
}

export async function batchDeleteGlobalModels(ids: string[]) {
  const result = await requestData<BatchDeleteGlobalModelsResponse>(
    axios.post(endpoints.adminModels.globalBatchDelete, { ids })
  );
  await mutateGlobalModels();
  return result;
}

export async function getGlobalModelProviders(id: string) {
  return requestData<GlobalModelProvidersResponse>(
    axios.get(endpoints.adminModels.globalProviders(id))
  );
}

export async function getModelCatalog() {
  return requestData<ModelCatalogResponse>(axios.get(endpoints.adminModels.catalog));
}

export function useUserModelCatalog() {
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<GlobalModelListResponse>
  >(
    endpoints.models.catalog,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const catalog = data ? requireApiData(data) : undefined;
    return {
      data: catalog,
      items: catalog?.models ?? [],
      total: catalog?.total ?? 0,
      isLoading,
      error,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function getModelsDevData() {
  return requestData<ModelsDevData>(axios.get(endpoints.adminModels.external));
}

export async function getModelsDevList(officialOnly = true) {
  const data = await getModelsDevData();
  const items = flattenModelsDevData(data);
  return officialOnly ? items.filter((item) => item.official) : items;
}

function flattenModelsDevData(data: ModelsDevData) {
  return Object.entries(data).flatMap(([providerId, provider]) =>
    normalizeProviderModels(providerId, provider)
  );
}

function normalizeProviderModels(providerId: string, provider: ModelsDevData[string]) {
  return Object.entries(provider.models ?? {})
    .map(([modelId, model]) => normalizeModel(providerId, provider, modelId, model))
    .sort(compareModelsDevItems);
}

function normalizeModel(
  providerId: string,
  provider: ModelsDevData[string],
  modelId: string,
  model: NonNullable<ModelsDevData[string]['models']>[string]
): ModelsDevModelItem {
  const inputModalities = model.modalities?.input ?? model.input ?? [];
  const outputModalities = model.modalities?.output ?? model.output ?? [];
  const inputPrice = model.cost?.input;

  return {
    providerId,
    providerName: provider.name ?? providerId,
    modelId,
    modelName: model.name ?? model.id ?? modelId,
    family: model.family,
    inputPrice,
    outputPrice: model.cost?.output,
    cacheCreationPrice: model.cost?.cache_write ?? aetherCacheCreationPrice(inputPrice),
    cacheReadPrice: model.cost?.cache_read ?? aetherCacheReadPrice(inputPrice),
    cache1hCreationPrice: aetherCache1hCreationPrice(inputPrice),
    contextLimit: model.limit?.context,
    outputLimit: model.limit?.output,
    supportsVision: inputModalities.includes('image'),
    supportsToolCall: model.tool_call === true,
    supportsReasoning: model.reasoning === true,
    supportsStructuredOutput: model.structured_output === true,
    supportsTemperature: model.temperature === true,
    supportsAttachment: model.attachment === true,
    openWeights: model.open_weights === true,
    deprecated: model.deprecated === true,
    official: provider.official === true,
    knowledgeCutoff: model.knowledge,
    releaseDate: model.release_date,
    inputModalities,
    outputModalities,
  };
}

function compareModelsDevItems(left: ModelsDevModelItem, right: ModelsDevModelItem) {
  const providerCompare = left.providerName.localeCompare(right.providerName);
  if (providerCompare !== 0) return providerCompare;

  const leftTime = left.releaseDate ? new Date(left.releaseDate).getTime() : 0;
  const rightTime = right.releaseDate ? new Date(right.releaseDate).getTime() : 0;
  if (leftTime !== rightTime) return rightTime - leftTime;

  return left.modelName.localeCompare(right.modelName);
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

async function mutateGlobalModels() {
  await mutate((key) => key === endpoints.adminModels.global || isGlobalModelKey(key));
}

function isGlobalModelKey(key: unknown) {
  return Array.isArray(key) && key[0] === endpoints.adminModels.global;
}
