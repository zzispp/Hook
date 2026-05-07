'use client';

import type { ApiEnvelope } from './utils';

import axios, { endpoints } from 'src/lib/axios';

import { trimCredential } from './validation';
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
  access_token: string;
  refresh_token: string;
};

/** **************************************
 * Sign in
 *************************************** */
export const signInWithPassword = async ({ identifier, password }: SignInParams): Promise<void> => {
  try {
    const params = {
      identifier: trimCredential(identifier),
      password: trimCredential(password),
    };

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
    username: trimCredential(username),
    email: trimCredential(email),
    password: trimCredential(password),
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

  if (!session.access_token || !session.refresh_token) {
    throw new Error('Auth tokens not found in response');
  }

  return session;
}
