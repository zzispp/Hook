'use client';

import type { ApiEnvelope } from 'src/types/rbac';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

export type CaptchaConfig = {
  login_captcha_enabled: boolean;
  registration_captcha_enabled: boolean;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useCaptchaConfig() {
  const { data, isLoading, error, isValidating, mutate } = useSWR<ApiEnvelope<CaptchaConfig>>(
    endpoints.captcha.config,
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
