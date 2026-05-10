'use client';

import type { ApiEnvelope } from './utils';
import type { AuthState } from '../../types';

import { useSetState } from 'minimal-shared/hooks';
import { useMemo, useEffect, useCallback } from 'react';

import axios, { endpoints, HTTP_UNAUTHORIZED, isApiRequestError } from 'src/lib/axios';

import { AuthContext } from '../auth-context';
import { setSession, isValidToken, requireApiData } from './utils';
import { JWT_STORAGE_KEY, JWT_REFRESH_STORAGE_KEY } from './constant';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

type TokenPairResponse = {
  access_token: string;
  refresh_token: string;
};

type MeResponse = {
  user: {
    id: string;
    username: string;
    email: string;
    role: string;
    is_active: boolean;
    auth_source: string;
    email_verified: boolean;
    system: boolean;
  };
};

export function AuthProvider({ children }: Props) {
  const { state, setState } = useSetState<AuthState>({
    user: null,
    error: null,
    loading: true,
  });

  const checkUserSession = useCallback(async () => {
    const session = await resolveSession();

    if (!session) {
      await setSession(null);
      setState({ user: null, error: null, loading: false });
      return;
    }

    const me = await resolveCurrentUser();

    if (!me) {
      await setSession(null);
      setState({ user: null, error: null, loading: false });
      return;
    }

    const { user } = me;

    setState({
      user: {
        ...user,
        access_token: session.access_token,
        displayName: user.username,
      },
      error: null,
      loading: false,
    });
  }, [setState]);

  useEffect(() => {
    checkUserSession().catch((error: Error) => {
      console.error(error);
      setState({ error, loading: false });
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ----------------------------------------------------------------------

  const checkAuthenticated = state.user ? 'authenticated' : 'unauthenticated';

  const status = state.loading ? 'loading' : checkAuthenticated;

  const memoizedValue = useMemo(
    () => ({
      user: state.user ? { ...state.user, role: state.user?.role ?? 'admin' } : null,
      checkUserSession,
      error: state.error,
      loading: status === 'loading',
      authenticated: status === 'authenticated',
      unauthenticated: status === 'unauthenticated',
    }),
    [checkUserSession, state.error, state.user, status]
  );

  if (state.error) {
    throw state.error;
  }

  return <AuthContext value={memoizedValue}>{children}</AuthContext>;
}

async function resolveSession() {
  const access_token = localStorage.getItem(JWT_STORAGE_KEY);
  const refresh_token = localStorage.getItem(JWT_REFRESH_STORAGE_KEY);

  if (access_token && refresh_token && isValidToken(access_token)) {
    const session = { access_token, refresh_token };
    await setSession(session);
    return session;
  }

  if (!refresh_token || !isValidToken(refresh_token)) {
    return null;
  }

  const session = await refreshSession(refresh_token);

  if (!session) {
    return null;
  }

  await setSession(session);

  return session;
}

async function refreshSession(refresh_token: string) {
  try {
    const res = await axios.post<ApiEnvelope<TokenPairResponse>>(endpoints.auth.refresh, { refresh_token });
    if (isUnauthorizedPayload(res.data)) {
      return null;
    }
    return requireApiData<TokenPairResponse>(res.data);
  } catch (error) {
    if (isUnauthorizedRequest(error)) {
      return null;
    }

    throw error;
  }
}

async function resolveCurrentUser() {
  try {
    const res = await axios.get<ApiEnvelope<MeResponse>>(endpoints.auth.me);
    if (isUnauthorizedPayload(res.data)) {
      return null;
    }
    return requireApiData<MeResponse>(res.data);
  } catch (error) {
    if (isUnauthorizedRequest(error)) {
      return null;
    }

    throw error;
  }
}

function isUnauthorizedRequest(error: unknown) {
  return isApiRequestError(error) && error.status === HTTP_UNAUTHORIZED;
}

function isUnauthorizedPayload(payload: unknown) {
  if (!isApiEnvelope(payload) || payload.success) {
    return false;
  }

  return payload.message.toLowerCase() === 'unauthorized';
}

function isApiEnvelope(payload: unknown): payload is ApiEnvelope<unknown> {
  return (
    typeof payload === 'object' &&
    payload !== null &&
    'success' in payload &&
    typeof payload.success === 'boolean' &&
    'message' in payload &&
    typeof payload.message === 'string'
  );
}
