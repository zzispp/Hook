'use client';

import type { Key } from 'swr';
import type { ApiEnvelope } from 'src/types/rbac';
import type {
  WalletBalanceResponse,
  AdminWalletListResponse,
  AdminWalletRechargeInput,
  AdminWalletLedgerResponse,
  AdminWalletAdjustmentInput,
  WalletTransactionsResponse,
  AdminWalletRechargeResponse,
  WalletLedgerEntriesResponse,
  AdminWalletAdjustmentResponse,
  AdminWalletTransactionsResponse,
  WalletDailyUsageDetailsResponse,
  AdminWalletLedgerEntriesResponse,
  AdminWalletDailyUsageDetailsResponse,
  AdminWalletConsumptionSummaryResponse,
  AdminWalletLedgerEntriesForWalletResponse,
} from 'src/types/wallet';

import useSWR from 'swr';
import { useMemo } from 'react';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { pageQuery, requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useWalletBalance() {
  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<WalletBalanceResponse>
  >(endpoints.wallet.balance, fetcher, swrOptions);

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    const balance = data?.success ? requireApiData(data) : undefined;
    return {
      data: balance,
      wallet: balance?.wallet,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

export function useWalletTransactions(page: number, pageSize: number) {
  const key = [
    endpoints.wallet.transactions,
    { params: { page: page + 1, page_size: pageSize } },
  ] as const;

  const { data, isLoading, error, isValidating, mutate } = useSWR<
    ApiEnvelope<WalletTransactionsResponse>
  >(key, fetcher, swrOptions);

  return useMemo(() => {
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    const transactions = data?.success ? requireApiData(data) : undefined;
    return {
      data: transactions,
      items: transactions?.items ?? [],
      total: transactions?.total ?? 0,
      wallet: transactions?.wallet,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: mutate,
    };
  }, [data, error, isLoading, isValidating, mutate]);
}

export function useWalletLedgerEntries(page: number, pageSize: number, filters: WalletLedgerEntryFilters = {}) {
  const key = [endpoints.wallet.ledgerEntries, { params: ledgerParams(page, pageSize, filters) }] as const;
  return useWalletResource<WalletLedgerEntriesResponse>(key);
}

export function useWalletDailyModelUsage(date: string | null, page: number, pageSize: number) {
  const key = date
    ? ([endpoints.wallet.dailyModelUsage, { params: dailyUsageParams(date, page, pageSize) }] as const)
    : null;
  return useWalletResource<WalletDailyUsageDetailsResponse>(key);
}

export type AdminWalletFilters = {
  search?: string;
  status?: string;
};

export type AdminWalletLedgerFilters = {
  category?: string;
  reason_code?: string;
  owner_type?: string;
};

export type WalletLedgerRangePreset = 'all' | 'today' | 'last7days' | 'last30days' | 'custom';

export type WalletLedgerEntryFilters = {
  search?: string;
  category?: string;
  reason_code?: string;
  direction?: string;
  balance_type?: string;
  link_type?: string;
  owner_type?: string;
  range_preset?: WalletLedgerRangePreset;
  start_date?: string;
  end_date?: string;
};

export function useAdminWallets(page: number, pageSize: number, filters: AdminWalletFilters = {}) {
  const key = [endpoints.adminWallets.list, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useWalletResource<AdminWalletListResponse>(key);
}

export function useAdminWalletLedger(page: number, pageSize: number, filters: AdminWalletLedgerFilters = {}) {
  const key = [endpoints.adminWallets.ledger, { params: { ...pageQuery(page, pageSize), ...filters } }] as const;
  return useWalletResource<AdminWalletLedgerResponse>(key);
}

export function useAdminWalletLedgerEntries(page: number, pageSize: number, filters: WalletLedgerEntryFilters = {}, enabled = true) {
  const key = enabled
    ? ([endpoints.adminWallets.ledgerEntries, { params: ledgerParams(page, pageSize, filters) }] as const)
    : null;
  return useWalletResource<AdminWalletLedgerEntriesResponse>(key);
}

export function useAdminWalletConsumptionSummary(page: number, pageSize: number, filters: WalletLedgerEntryFilters = {}, enabled = true) {
  const key = enabled
    ? ([endpoints.adminWallets.consumptionSummary, { params: ledgerParams(page, pageSize, filters) }] as const)
    : null;
  return useWalletResource<AdminWalletConsumptionSummaryResponse>(key);
}

export function useAdminWalletTransactions(walletId: string | null, page: number, pageSize: number) {
  const key = walletId
    ? ([endpoints.adminWallets.transactions(walletId), { params: pageQuery(page, pageSize) }] as const)
    : null;
  return useWalletResource<AdminWalletTransactionsResponse>(key);
}

export function useAdminWalletLedgerEntriesForWallet(
  walletId: string | null,
  page: number,
  pageSize: number,
  filters: WalletLedgerEntryFilters = {}
) {
  const key = walletId
    ? ([endpoints.adminWallets.ledgerEntriesForWallet(walletId), { params: ledgerParams(page, pageSize, filters) }] as const)
    : null;
  return useWalletResource<AdminWalletLedgerEntriesForWalletResponse>(key);
}

export function useAdminWalletDailyModelUsage(walletId: string | null, date: string | null, page: number, pageSize: number) {
  const key =
    walletId && date
      ? ([endpoints.adminWallets.dailyModelUsageForWallet(walletId), { params: dailyUsageParams(date, page, pageSize) }] as const)
      : null;
  return useWalletResource<AdminWalletDailyUsageDetailsResponse>(key);
}

export function useAdminUserWalletBalance(userId: string | null) {
  const key = userId ? endpoints.adminWallets.userBalance(userId) : null;
  return useWalletResource<WalletBalanceResponse>(key);
}

export async function adjustAdminWallet(walletId: string, payload: AdminWalletAdjustmentInput) {
  const response = await axios.post<ApiEnvelope<AdminWalletAdjustmentResponse>>(
    endpoints.adminWallets.adjust(walletId),
    payload
  );
  return requireApiData(response.data);
}

export async function rechargeAdminWallet(walletId: string, payload: AdminWalletRechargeInput) {
  const response = await axios.post<ApiEnvelope<AdminWalletRechargeResponse>>(
    endpoints.adminWallets.recharge(walletId),
    payload
  );
  return requireApiData(response.data);
}

function ledgerParams(page: number, pageSize: number, filters: WalletLedgerEntryFilters) {
  return {
    ...pageQuery(page, pageSize),
    tz_offset_minutes: timezoneOffsetMinutes(),
    ...filters,
  };
}

function dailyUsageParams(date: string, page: number, pageSize: number) {
  return {
    ...pageQuery(page, pageSize),
    tz_offset_minutes: timezoneOffsetMinutes(),
    date,
  };
}

function timezoneOffsetMinutes() {
  return -new Date().getTimezoneOffset();
}

function useWalletResource<T>(key: Key) {
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
