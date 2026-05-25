'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  PaymentChannel,
  RechargePackage,
  RechargePackageInput,
  RechargeOrderCreateInput,
  RechargeOrderListResponse,
  PaymentChannelUpdateInput,
  RechargePackageListResponse,
  UserRechargePackageListResponse,
} from 'src/types/recharge';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type RechargePackageFilters = {
  search?: string;
  status?: string;
};

export type RechargeOrderFilters = {
  search?: string;
  status?: string;
};

export function useRechargePackages(
  page: number,
  pageSize: number,
  filters: RechargePackageFilters = {}
) {
  const key = [
    endpoints.adminRecharges.packages,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useRechargeResource<RechargePackageListResponse>(key);
}

export function useRechargeOrders(
  page: number,
  pageSize: number,
  filters: RechargeOrderFilters = {}
) {
  const key = [
    endpoints.adminRecharges.orders,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useRechargeResource<RechargeOrderListResponse>(key);
}

export function usePaymentChannels() {
  return useRechargeResource<PaymentChannel[]>(endpoints.adminRecharges.paymentChannels);
}

export function useUserRechargePackages(page = 1, pageSize = 50) {
  const key = [endpoints.recharges.packages, { params: pageQuery(page, pageSize) }] as const;
  return useRechargeResource<UserRechargePackageListResponse>(key);
}

export function useUserRechargeOrders(page = 1, pageSize = 5) {
  const key = [endpoints.recharges.orders, { params: pageQuery(page, pageSize) }] as const;
  return useRechargeResource<RechargeOrderListResponse>(key);
}

export async function createRechargePackage(payload: RechargePackageInput) {
  const response = await axios.post<ApiEnvelope<RechargePackage>>(
    endpoints.adminRecharges.packages,
    payload
  );
  return requireApiData(response.data);
}

export async function updateRechargePackage(id: string, payload: RechargePackageInput) {
  const response = await axios.patch<ApiEnvelope<RechargePackage>>(
    endpoints.adminRecharges.package(id),
    payload
  );
  return requireApiData(response.data);
}

export async function updatePaymentChannel(code: string, payload: PaymentChannelUpdateInput) {
  const response = await axios.patch<ApiEnvelope<PaymentChannel>>(
    endpoints.adminRecharges.paymentChannel(code),
    payload
  );
  return requireApiData(response.data);
}

export async function createUserRechargeOrder(payload: RechargeOrderCreateInput) {
  const response = await axios.post<ApiEnvelope<RechargeOrderListResponse['items'][number]>>(
    endpoints.recharges.orders,
    payload
  );
  return requireApiData(response.data);
}

function useRechargeResource<T>(key: Key) {
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
