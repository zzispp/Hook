'use client';

import type { SystemUser, ApiEnvelope, UserIdentitySummary } from 'src/types/rbac';

import { useMemo } from 'react';
import useSWR, { mutate } from 'swr';

import axios, { fetcher, endpoints } from 'src/lib/axios';

import { requireApiData } from './rbac';

type AccountProfileResponse = {
  user: SystemUser;
};

type PasswordChangePayload = {
  emailVerificationCode: string;
  password: string;
};

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useAccountProfile() {
  const { data, isLoading, error, isValidating, mutate: revalidate } = useSWR<
    ApiEnvelope<AccountProfileResponse>
  >(endpoints.account.profile, fetcher, swrOptions);

  return useMemo(() => {
    const profile = data?.success ? requireApiData(data) : undefined;
    const apiError = data && !data.success ? new Error(data.message || 'Request failed') : undefined;

    return {
      data: profile?.user,
      isLoading,
      error: error ?? apiError,
      isValidating,
      refresh: revalidate,
    };
  }, [data, error, isLoading, isValidating, revalidate]);
}

export async function requestAccountPasswordEmailCode(lang: string) {
  await axios.post(endpoints.account.passwordEmailCode, { lang });
}

export async function changeAccountPassword(payload: PasswordChangePayload) {
  const response = await axios.post<ApiEnvelope<SystemUser>>(endpoints.account.passwordChange, {
    email_verification_code: payload.emailVerificationCode.trim(),
    password: payload.password.trim(),
  });
  const user = requireApiData(response.data);
  await mutate(endpoints.account.profile);
  await mutate(endpoints.auth.me);
  return user;
}

export async function deleteAccountIdentity(identityId: string) {
  await axios.delete(endpoints.account.identity(identityId));
  await mutate(endpoints.account.profile);
  await mutate(endpoints.account.identities);
  await mutate(endpoints.auth.me);
}

export async function getAccountIdentities() {
  const response = await axios.get<ApiEnvelope<UserIdentitySummary[]>>(endpoints.account.identities);
  return requireApiData(response.data);
}
