import { JWT_STORAGE_KEY, JWT_REFRESH_STORAGE_KEY } from './constant';

export const ACCESS_TOKEN_REFRESH_WINDOW_SECONDS = 60;

export type JwtSession = {
  access_token: string;
  refresh_token: string;
};

type JwtPayload = {
  exp?: number;
  [key: string]: unknown;
};

export function jwtDecode(token: string): JwtPayload | null {
  if (!token) {
    return null;
  }

  const parts = token.split('.');

  if (parts.length < 2) {
    throw new Error('Invalid token!');
  }

  const base64Url = parts[1];
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');

  return JSON.parse(atob(base64));
}

export function isValidToken(token: string) {
  const expiresAt = tokenExpiresAt(token);

  if (!expiresAt) {
    return false;
  }

  return expiresAt > currentTimestamp();
}

export function shouldRefreshAccessToken(token: string) {
  const expiresAt = tokenExpiresAt(token);

  if (!expiresAt) {
    return true;
  }

  return expiresAt <= currentTimestamp() + ACCESS_TOKEN_REFRESH_WINDOW_SECONDS;
}

export function storedSession(): JwtSession | null {
  if (!browserStorageAvailable()) {
    return null;
  }

  const accessToken = localStorage.getItem(JWT_STORAGE_KEY);
  const refreshToken = localStorage.getItem(JWT_REFRESH_STORAGE_KEY);

  if (!accessToken || !refreshToken) {
    return null;
  }

  return {
    access_token: accessToken,
    refresh_token: refreshToken,
  };
}

export function storeSession(session: JwtSession) {
  if (!browserStorageAvailable()) {
    return;
  }

  localStorage.setItem(JWT_STORAGE_KEY, session.access_token);
  localStorage.setItem(JWT_REFRESH_STORAGE_KEY, session.refresh_token);
}

export function clearStoredSession() {
  if (!browserStorageAvailable()) {
    return;
  }

  localStorage.removeItem(JWT_STORAGE_KEY);
  localStorage.removeItem(JWT_REFRESH_STORAGE_KEY);
}

function tokenExpiresAt(token: string) {
  try {
    const decoded = jwtDecode(token);
    return typeof decoded?.exp === 'number' ? decoded.exp : null;
  } catch (error) {
    console.error('Error during token validation:', error);
    return null;
  }
}

function currentTimestamp() {
  return Date.now() / 1000;
}

function browserStorageAvailable() {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}
