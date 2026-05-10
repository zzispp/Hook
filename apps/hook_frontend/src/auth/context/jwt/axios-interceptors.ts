import type { AxiosError, AxiosInstance, InternalAxiosRequestConfig } from 'axios';

import axios, { AxiosHeaders } from 'axios';

import { CONFIG } from 'src/global-config';

import { JWT_REFRESH_ENDPOINT } from './constant';
import {
  isValidToken,
  storeSession,
  storedSession,
  clearStoredSession,
  shouldRefreshAccessToken,
} from './session';

const HTTP_UNAUTHORIZED = 401;

type ApiEnvelope<T> = {
  success: boolean;
  message?: string;
  data?: T;
};

type TokenPairResponse = {
  access_token: string;
  refresh_token: string;
};

type RetriableRequestConfig = InternalAxiosRequestConfig & {
  _retry?: boolean;
};

let axiosClient: AxiosInstance | null = null;
let refreshPromise: Promise<TokenPairResponse | null> | null = null;

export function installJwtInterceptors(client: AxiosInstance) {
  axiosClient = client;
  client.interceptors.request.use(async (config) => {
    if (isRefreshRequest(config)) {
      return config;
    }

    const accessToken = await validAccessToken();
    setRequestAuthorization(config, accessToken);

    return config;
  });
}

export function setAccessTokenHeader(accessToken: string | null) {
  if (!axiosClient) {
    return;
  }

  if (accessToken) {
    axiosClient.defaults.headers.common.Authorization = `Bearer ${accessToken}`;
    return;
  }

  delete axiosClient.defaults.headers.common.Authorization;
}

export async function retryAfterUnauthorized(error: unknown) {
  if (!axiosClient || !canRetryAfterUnauthorized(error)) {
    return null;
  }

  const config = error.config as RetriableRequestConfig;
  const session = await refreshStoredSession();

  if (!session) {
    return null;
  }

  config._retry = true;
  setRequestAuthorization(config, session.access_token);
  return axiosClient.request(config);
}

function canRetryAfterUnauthorized(error: unknown): error is AxiosError {
  if (!axios.isAxiosError(error) || error.response?.status !== HTTP_UNAUTHORIZED) {
    return false;
  }

  const config = error.config as RetriableRequestConfig | undefined;
  return Boolean(config && !config._retry && !isRefreshRequest(config) && hasStaleAccessToken(config));
}

async function validAccessToken() {
  const session = storedSession();

  if (!session) {
    return null;
  }

  if (!isValidToken(session.refresh_token)) {
    clearBrowserSession();
    return null;
  }

  if (isValidToken(session.access_token) && !shouldRefreshAccessToken(session.access_token)) {
    return session.access_token;
  }

  const refreshed = await refreshStoredSession();
  return refreshed?.access_token ?? null;
}

async function refreshStoredSession() {
  const session = storedSession();

  if (!session || !isValidToken(session.refresh_token)) {
    clearBrowserSession();
    return null;
  }

  refreshPromise ??= requestTokenPair(session.refresh_token).finally(() => {
    refreshPromise = null;
  });

  return refreshPromise;
}

async function requestTokenPair(refreshToken: string) {
  try {
    const response = await axios.post<ApiEnvelope<TokenPairResponse>>(
      JWT_REFRESH_ENDPOINT,
      { refresh_token: refreshToken },
      { baseURL: CONFIG.serverUrl, headers: { 'Content-Type': 'application/json' } }
    );
    if (isUnauthorizedPayload(response.data)) {
      clearBrowserSession();
      return null;
    }
    const session = requireApiData(response.data);
    storeSession(session);
    setAccessTokenHeader(session.access_token);
    return session;
  } catch (error) {
    if (axios.isAxiosError(error) && error.response?.status === HTTP_UNAUTHORIZED) {
      clearBrowserSession();
      return null;
    }

    throw error;
  }
}

function requireApiData<T>(payload: ApiEnvelope<T>) {
  if (!payload.success || payload.data === undefined || payload.data === null) {
    throw new Error(payload.message || 'Request failed');
  }

  return payload.data;
}

function isUnauthorizedPayload(payload: ApiEnvelope<unknown>) {
  return !payload.success && payload.message?.toLowerCase() === 'unauthorized';
}

function clearBrowserSession() {
  clearStoredSession();
  setAccessTokenHeader(null);
}

function setRequestAuthorization(config: InternalAxiosRequestConfig, accessToken: string | null) {
  const headers = AxiosHeaders.from(config.headers);

  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`);
  } else {
    headers.delete('Authorization');
  }

  config.headers = headers;
}

function hasStaleAccessToken(config: InternalAxiosRequestConfig) {
  const session = storedSession();
  const requestToken = requestAccessToken(config);

  if (!session || !requestToken || !isValidToken(session.refresh_token)) {
    return false;
  }

  return requestToken !== session.access_token || !isValidToken(requestToken) || shouldRefreshAccessToken(requestToken);
}

function requestAccessToken(config: InternalAxiosRequestConfig) {
  const value = AxiosHeaders.from(config.headers).get('Authorization');

  if (typeof value !== 'string' || !value.startsWith('Bearer ')) {
    return null;
  }

  return value.slice('Bearer '.length);
}

function isRefreshRequest(config: Pick<InternalAxiosRequestConfig, 'url'>) {
  return normalizedPath(config.url) === JWT_REFRESH_ENDPOINT;
}

function normalizedPath(url?: string) {
  if (!url) {
    return '';
  }

  return url.startsWith('http') ? new URL(url).pathname : url.split('?')[0];
}
