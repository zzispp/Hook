'use client';

import type { ApiEnvelope } from 'src/types/rbac';
import type {
  SystemSettings,
  ExchangeRateResponse,
  SystemSettingsUpdate,
} from 'src/types/system-setting';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useSystemSettings(enabled = true) {
  const {
    data,
    isLoading,
    error,
    isValidating,
    mutate: revalidate,
  } = useSWR<ApiEnvelope<SystemSettings>>(
    enabled ? endpoints.adminSettings.system : null,
    fetcher,
    swrOptions
  );

  return useMemo(() => {
    const apiError =
      data && !data.success ? new Error(data.message || 'Request failed') : undefined;
    return {
      data: enabled && data?.success ? requireApiData(data) : undefined,
      isLoading: enabled ? isLoading : false,
      error: error ?? apiError,
      isValidating: enabled ? isValidating : false,
      refresh: revalidate,
    };
  }, [data, enabled, error, isLoading, isValidating, revalidate]);
}

export async function updateSystemSettings(payload: SystemSettingsUpdate) {
  const response = await axios.patch<ApiEnvelope<SystemSettings>>(
    endpoints.adminSettings.system,
    payload
  );
  const settings = requireApiData(response.data);
  await mutate(endpoints.adminSettings.system);
  return settings;
}

export async function updateSchedulingMode(schedulingMode: SystemSettings['scheduling_mode']) {
  return updateSystemSettings({ scheduling_mode: schedulingMode });
}

export function useUsdCnyExchangeRate(enabled: boolean) {
  const {
    data,
    isLoading,
    error,
    isValidating,
    mutate: revalidate,
  } = useSWR<ApiEnvelope<ExchangeRateResponse>>(
    enabled ? endpoints.adminSettings.exchangeRate : null,
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
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}
