import type { JwtSession } from './session';

import { setAccessTokenHeader } from './axios-interceptors';
import { jwtDecode, storeSession, isValidToken, clearStoredSession } from './session';

// ----------------------------------------------------------------------

export type ApiEnvelope<T> = {
  success: boolean;
  message: string;
  data?: T;
};

export { jwtDecode, isValidToken };
export type { JwtSession };

// ----------------------------------------------------------------------

export function requireApiData<T>(payload: ApiEnvelope<T>): T {
  if (!payload.success) {
    throw new Error(payload.message || 'Request failed');
  }

  if (payload.data === undefined || payload.data === null) {
    throw new Error('Response data not found');
  }

  return payload.data;
}

// ----------------------------------------------------------------------

export async function setSession(session: JwtSession | null) {
  try {
    if (session) {
      assertSession(session);
      const decodedToken = jwtDecode(session.access_token);

      if (!decodedToken || !('exp' in decodedToken)) {
        throw new Error('Invalid access token!');
      }

      storeSession(session);
      setAccessTokenHeader(session.access_token);
    } else {
      clearStoredSession();
      setAccessTokenHeader(null);
    }
  } catch (error) {
    console.error('Error during set session:', error);
    throw error;
  }
}

function assertSession(session: JwtSession) {
  if (!session.access_token || !session.refresh_token) {
    throw new Error('Auth tokens not found in response');
  }
}
