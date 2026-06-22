'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ProviderApiKey,
  ProviderKeyModelMappingsUpdate,
  ProviderKeyModelMappingsForKeyResponse,
} from 'src/types/provider';
import type {
  ProviderQuickImportRelinkRequest,
  ProviderQuickImportCommitResponse,
  ProviderQuickImportPreviewResponse,
  ProviderQuickImportBindCommitRequest,
  ProviderQuickImportBindCommitResponse,
  ProviderQuickImportBindPreviewRequest,
  ProviderQuickImportResolutionResponse,
  ProviderQuickImportBindPreviewResponse,
  ProviderQuickImportAppendCommitRequest,
  ProviderQuickImportAppendPreviewRequest,
  ProviderQuickImportSyncEventDetailResponse,
} from 'src/types/provider-quick-import';

import { mutate } from 'swr';

import axios, { endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export async function previewProviderQuickImportAppend(
  providerId: string,
  payload: ProviderQuickImportAppendPreviewRequest = {}
) {
  return requestData<ProviderQuickImportPreviewResponse>(
    axios.post(endpoints.adminProviders.quickImportAppendPreview(providerId), payload)
  );
}

export async function commitProviderQuickImportAppend(
  providerId: string,
  payload: ProviderQuickImportAppendCommitRequest
) {
  const response = await requestData<ProviderQuickImportCommitResponse>(
    axios.post(endpoints.adminProviders.quickImportAppendCommit(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  await mutateProviders();
  return response;
}

export async function previewProviderQuickImportBind(
  providerId: string,
  payload: ProviderQuickImportBindPreviewRequest
) {
  return requestData<ProviderQuickImportBindPreviewResponse>(
    axios.post(endpoints.adminProviders.quickImportBindPreview(providerId), payload)
  );
}

export async function commitProviderQuickImportBind(
  providerId: string,
  payload: ProviderQuickImportBindCommitRequest
) {
  const response = await requestData<ProviderQuickImportBindCommitResponse>(
    axios.post(endpoints.adminProviders.quickImportBindCommit(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  await mutateProviders();
  return response;
}

export async function getProviderQuickImportResolution(providerId: string, keyId: string) {
  return requestData<ProviderQuickImportResolutionResponse>(
    axios.get(endpoints.adminProviders.keyQuickImportResolution(providerId, keyId))
  );
}

export async function acceptProviderQuickImportCurrent(providerId: string, keyId: string) {
  const apiKey = await requestData<ProviderApiKey>(
    axios.post(endpoints.adminProviders.keyQuickImportAcceptCurrent(providerId, keyId))
  );
  await mutateProviderChildren(providerId);
  return apiKey;
}

export async function relinkProviderQuickImportKey(
  providerId: string,
  keyId: string,
  payload: ProviderQuickImportRelinkRequest
) {
  const apiKey = await requestData<ProviderApiKey>(
    axios.post(endpoints.adminProviders.keyQuickImportRelink(providerId, keyId), payload)
  );
  await mutateProviderChildren(providerId);
  return apiKey;
}

export async function getProviderKeyModelMappingsForKey(providerId: string, keyId: string) {
  return requestData<ProviderKeyModelMappingsForKeyResponse>(
    axios.get(endpoints.adminProviders.keyModelMappingsByKey(providerId, keyId))
  );
}

export async function updateProviderKeyModelMappings(
  providerId: string,
  keyId: string,
  payload: ProviderKeyModelMappingsUpdate
) {
  const response = await requestData<ProviderKeyModelMappingsForKeyResponse>(
    axios.put(endpoints.adminProviders.keyModelMappingsByKey(providerId, keyId), payload)
  );
  await mutateProviderChildren(providerId);
  return response;
}

export async function getProviderQuickImportSyncEventDetail(id: string) {
  return requestData<ProviderQuickImportSyncEventDetailResponse>(
    axios.get(endpoints.adminProviders.quickImportSyncEventDetail(id))
  );
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function mutateProviders() {
  await mutate((key) => key === endpoints.adminProviders.list || isProviderListKey(key));
}

async function mutateProviderChildren(providerId: string) {
  await mutate((key) => isProviderChildKey(key, providerId) || isProviderPriorityKeysKey(key, providerId));
}

function isProviderListKey(key: unknown) {
  return Array.isArray(key) && key[0] === endpoints.adminProviders.list;
}

function isProviderChildKey(key: unknown, providerId: string) {
  return typeof key === 'string' && key.startsWith(`/api/admin/providers/${providerId}/`);
}

function isProviderPriorityKeysKey(key: unknown, providerId: string) {
  return (
    Array.isArray(key) &&
    key[0] === 'provider-priority-keys' &&
    Array.isArray(key[1]) &&
    key[1].includes(providerId)
  );
}
