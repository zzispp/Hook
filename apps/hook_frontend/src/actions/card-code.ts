'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  CardCodeType,
  CardCodeTypeInput,
  CardCodeRedeemInput,
  CardCodeListResponse,
  CardCodeGenerateInput,
  CardCodeTypeListResponse,
  CardCodeGenerateResponse,
  CardCodeBatchStatusInput,
  CardCodeBatchStatusResponse,
} from 'src/types/card-code';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type CardCodeFilters = {
  search?: string;
  status?: string;
  type_id?: string;
};

export type CardCodeTypeFilters = {
  search?: string;
  status?: string;
};

export function useCardCodes(page: number, pageSize: number, filters: CardCodeFilters = {}) {
  const key = [endpoints.cardCodes.list, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useCardCodeResource<CardCodeListResponse>(key);
}

export function useCardCodeTypes(page: number, pageSize: number, filters: CardCodeTypeFilters = {}) {
  const key = [endpoints.cardCodes.types, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useCardCodeResource<CardCodeTypeListResponse>(key);
}

export async function fetchCardCodes(page: number, pageSize: number, filters: CardCodeFilters = {}) {
  const response = await axios.get<ApiEnvelope<CardCodeListResponse>>(endpoints.cardCodes.list, {
    params: { ...pageQuery(page, pageSize), ...filters },
  });
  return requireApiData(response.data);
}

export async function createCardCodeType(payload: CardCodeTypeInput) {
  const response = await axios.post<ApiEnvelope<CardCodeType>>(endpoints.cardCodes.types, payload);
  return requireApiData(response.data);
}

export async function updateCardCodeType(id: string, payload: CardCodeTypeInput) {
  const response = await axios.patch<ApiEnvelope<CardCodeType>>(
    endpoints.cardCodes.type(id),
    payload
  );
  return requireApiData(response.data);
}

export async function generateCardCodes(payload: CardCodeGenerateInput) {
  const response = await axios.post<ApiEnvelope<CardCodeGenerateResponse>>(
    endpoints.cardCodes.generate,
    payload
  );
  return requireApiData(response.data);
}

export async function batchUpdateCardCodes(payload: CardCodeBatchStatusInput) {
  const response = await axios.post<ApiEnvelope<CardCodeBatchStatusResponse>>(
    endpoints.cardCodes.batchStatus,
    payload
  );
  return requireApiData(response.data);
}

export async function redeemCardCode(payload: CardCodeRedeemInput) {
  const response = await axios.post<ApiEnvelope<unknown>>(endpoints.cardCodes.redeem, payload);
  return requireApiData(response.data);
}

function useCardCodeResource<T>(key: Key) {
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<T>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    return {
      data: data?.success ? requireApiData(data) : undefined,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}
