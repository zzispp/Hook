'use client';

import type { ApiEnvelope } from './utils';

import axios, { endpoints } from 'src/lib/axios';

import { setSession, requireApiData } from './utils';

// ----------------------------------------------------------------------

export type SignInParams = {
  identifier: string;
  password: string;
};

export type SignUpParams = {
  username: string;
  email: string;
  password: string;
};

type AuthSessionResponse = {
  accessToken: string;
  refreshToken: string;
};

/** **************************************
 * Sign in
 *************************************** */
export const signInWithPassword = async ({ identifier, password }: SignInParams): Promise<void> => {
  try {
    const params = { identifier, password };

    const res = await axios.post(endpoints.auth.signIn, params);

    await setSession(requireAuthSession(res.data));
  } catch (error) {
    console.error('Error during sign in:', error);
    throw error;
  }
};

/** **************************************
 * Sign up
 *************************************** */
export const signUp = async ({
  username,
  email,
  password,
}: SignUpParams): Promise<void> => {
  const params = {
    username,
    email,
    password,
    role: 'user',
    status: 'enabled',
  };

  try {
    const res = await axios.post(endpoints.auth.signUp, params);

    await setSession(requireAuthSession(res.data));
  } catch (error) {
    console.error('Error during sign up:', error);
    throw error;
  }
};

/** **************************************
 * Sign out
 *************************************** */
export const signOut = async (): Promise<void> => {
  try {
    await setSession(null);
  } catch (error) {
    console.error('Error during sign out:', error);
    throw error;
  }
};

function requireAuthSession(payload: ApiEnvelope<AuthSessionResponse>): AuthSessionResponse {
  const session = requireApiData<AuthSessionResponse>(payload);

  if (!session.accessToken || !session.refreshToken) {
    throw new Error('Auth tokens not found in response');
  }

  return session;
}
