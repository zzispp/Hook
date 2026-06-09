'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  PaymentChannel,
  RechargePackage,
  RechargePackageInput,
  PublicPaymentChannel,
  RechargeOrderCreateInput,
  PaymentChannelUpdateInput,
  RechargeOrderListResponse,
  RechargeOrderCreateResponse,
  RechargePackageListResponse,
  UserRechargePackageListResponse,
  PaymentCallbackRecordListResponse,
} from 'src/types/recharge';

import { useMemo } from 'react';
import useSWR, { mutate as mutateCache } from 'swr';

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

export type PaymentCallbackFilters = {
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

export function usePaymentCallbacks(
  page: number,
  pageSize: number,
  filters: PaymentCallbackFilters = {}
) {
  const key = [
    endpoints.adminRecharges.paymentCallbacks,
    { params: { ...pageQuery(page, pageSize), ...filters } },
  ] as const;
  return useRechargeResource<PaymentCallbackRecordListResponse>(key);
}

export function usePaymentChannels() {
  return useRechargeResource<PaymentChannel[]>(endpoints.adminRecharges.paymentChannels);
}

export function useUserPaymentChannels() {
  return useRechargeResource<PublicPaymentChannel[]>(endpoints.recharges.paymentChannels);
}

export function useUserRechargePackages(page = 0, pageSize = 50) {
  const key = [endpoints.recharges.packages, { params: pageQuery(page, pageSize) }] as const;
  return useRechargeResource<UserRechargePackageListResponse>(key);
}

export function useUserRechargeOrders(page = 0, pageSize = 5) {
  const key = userRechargeOrdersKey(page, pageSize);
  return useRechargeResource<RechargeOrderListResponse>(key);
}

export async function refreshUserRechargeOrdersPage(page = 0, pageSize = 5) {
  const key = userRechargeOrdersKey(page, pageSize);
  const response = await axios.get<ApiEnvelope<RechargeOrderListResponse>>(
    endpoints.recharges.orders,
    { params: pageQuery(page, pageSize) }
  );
  const data = requireApiData(response.data);
  await mutateCache(key, response.data, false);
  return data;
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
  const response = await axios.post<ApiEnvelope<RechargeOrderCreateResponse>>(
    endpoints.recharges.orders,
    payload
  );
  return requireApiData(response.data);
}

function userRechargeOrdersKey(page: number, pageSize: number) {
  return [endpoints.recharges.orders, { params: pageQuery(page, pageSize) }] as const;
}

function useRechargeResource<T>(key: Key) {
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<T>>(
    key,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const apiError =
      data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    return {
      data: data?.success ? requireApiData(data) : undefined,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}
