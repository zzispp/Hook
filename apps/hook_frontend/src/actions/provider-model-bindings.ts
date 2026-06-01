'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  ProviderModelBinding,
  ProviderModelBindingBatchUpdate,
} from 'src/types/provider';

import { mutate } from 'swr';

import axios from 'src/lib/axios';

import { requireApiData } from './rbac';

export async function batchUpdateProviderModels(
  providerId: string,
  payload: ProviderModelBindingBatchUpdate
) {
  const bindings = await requestData<ProviderModelBinding>(
    axios.post(batchUpdateEndpoint(providerId), payload)
  );
  await mutateProviderChildren(providerId);
  return bindings;
}

function batchUpdateEndpoint(providerId: string) {
  return `/api/admin/providers/${providerId}/models/batch-update`;
}

async function requestData<T>(request: Promise<{ data: ApiEnvelope<T[]> }>) {
  const response = await request;
  return requireApiData(response.data);
}

async function mutateProviderChildren(providerId: string) {
  await mutate((key) => typeof key === 'string' && key.startsWith(`/api/admin/providers/${providerId}/`));
}
