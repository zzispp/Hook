'use client';

import type { AuthState } from '../../types';

import { useSetState } from 'minimal-shared/hooks';
import { useMemo, useEffect, useCallback } from 'react';

import axios, { endpoints } from 'src/lib/axios';

import { AuthContext } from '../auth-context';
import { setSession, isValidToken, requireApiData } from './utils';
import { JWT_STORAGE_KEY, JWT_REFRESH_STORAGE_KEY } from './constant';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

type TokenPairResponse = {
  accessToken: string;
  refreshToken: string;
};

type MeResponse = {
  user: Record<string, unknown>;
};

export function AuthProvider({ children }: Props) {
  const { state, setState } = useSetState<AuthState>({ user: null, loading: true });

  const checkUserSession = useCallback(async () => {
    try {
      const session = await resolveSession();

      if (!session) {
        await setSession(null);
        setState({ user: null, loading: false });
        return;
      }

      const res = await axios.get(endpoints.auth.me);
      const { user } = requireApiData<MeResponse>(res.data);

      setState({ user: { ...user, accessToken: session.accessToken }, loading: false });
    } catch (error) {
      console.error(error);
      await setSession(null);
      setState({ user: null, loading: false });
    }
  }, [setState]);

  useEffect(() => {
    checkUserSession();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ----------------------------------------------------------------------

  const checkAuthenticated = state.user ? 'authenticated' : 'unauthenticated';

  const status = state.loading ? 'loading' : checkAuthenticated;

  const memoizedValue = useMemo(
    () => ({
      user: state.user ? { ...state.user, role: state.user?.role ?? 'admin' } : null,
      checkUserSession,
      loading: status === 'loading',
      authenticated: status === 'authenticated',
      unauthenticated: status === 'unauthenticated',
    }),
    [checkUserSession, state.user, status]
  );

  return <AuthContext value={memoizedValue}>{children}</AuthContext>;
}

async function resolveSession() {
  const accessToken = sessionStorage.getItem(JWT_STORAGE_KEY);
  const refreshToken = sessionStorage.getItem(JWT_REFRESH_STORAGE_KEY);

  if (accessToken && refreshToken && isValidToken(accessToken)) {
    await setSession({ accessToken, refreshToken });
    return { accessToken, refreshToken };
  }

  if (!refreshToken || !isValidToken(refreshToken)) {
    return null;
  }

  const res = await axios.post(endpoints.auth.refresh, { refreshToken });
  const session = requireApiData<TokenPairResponse>(res.data);

  await setSession(session);

  return session;
}
