'use client';

import type { ApiEnvelope } from './utils';
import type { SystemUser, IdentityProvider } from 'src/types/rbac';

import axios, { endpoints } from 'src/lib/axios';

import { trimCredential } from './validation';
import { setSession, requireApiData } from './utils';

// ----------------------------------------------------------------------

export type SignInParams = {
  identifier: string;
  password: string;
  captchaToken?: string;
};

export type SignUpParams = {
  username: string;
  email: string;
  password: string;
  emailVerificationCode?: string;
  captchaToken?: string;
};

export type RegistrationEmailCodeParams = {
  email: string;
  lang: string;
};

export type PasswordResetRequestParams = {
  email: string;
  lang: string;
  resetOrigin: string;
};

export type PasswordResetConfirmParams = {
  token: string;
  password: string;
};

type AuthSessionResponse = {
  user?: SystemUser;
  access_token: string;
  refresh_token: string;
};

export type OAuthCallbackResponse =
  | ({ status: 'authenticated' } & AuthSessionResponse)
  | {
      status: 'binding_required';
      binding_ticket: string;
      provider: IdentityProvider;
      email: string;
      username: string;
    };

export type WalletSignInResponse =
  | ({ status: 'authenticated' } & AuthSessionResponse)
  | {
      status: 'email_required';
      wallet_ticket: string;
      provider: IdentityProvider;
      address: string;
    };

export type WalletNonceParams = {
  provider: IdentityProvider;
  address: string;
  chainId?: number;
};

export type WalletSignInParams = WalletNonceParams & {
  message: string;
  signature: string;
};

/** **************************************
 * Sign in
 *************************************** */
export const signInWithPassword = async ({
  identifier,
  password,
  captchaToken,
}: SignInParams): Promise<void> => {
  try {
    const params = {
      identifier: trimCredential(identifier),
      password: trimCredential(password),
      ...(captchaToken && { captcha_token: captchaToken.trim() }),
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
  emailVerificationCode,
  captchaToken,
}: SignUpParams): Promise<void> => {
  const params = {
    username: trimCredential(username),
    email: trimCredential(email),
    password: trimCredential(password),
    ...(emailVerificationCode && { email_verification_code: emailVerificationCode.trim() }),
    ...(captchaToken && { captcha_token: captchaToken.trim() }),
  };

  try {
    const res = await axios.post(endpoints.auth.signUp, params);

    await setSession(requireAuthSession(res.data));
  } catch (error) {
    console.error('Error during sign up:', error);
    throw error;
  }
};

export const requestRegistrationEmailCode = async ({
  email,
  lang,
}: RegistrationEmailCodeParams): Promise<void> => {
  await axios.post(endpoints.auth.registrationEmailCode, {
    email: trimCredential(email),
    lang: trimCredential(lang),
  });
};

export async function startOAuth(provider: Extract<IdentityProvider, 'github' | 'google'>) {
  const res = await axios.get(endpoints.auth.oauthStart(provider));
  return requireApiData<{ authorization_url: string }>(res.data);
}

export async function completeOAuthCallback({
  provider,
  code,
  state,
}: {
  provider: Extract<IdentityProvider, 'github' | 'google'>;
  code: string;
  state: string;
}) {
  const res = await axios.get(endpoints.auth.oauthCallback(provider), {
    params: { code, state },
  });
  return requireApiData<OAuthCallbackResponse>(res.data);
}

export async function bindOAuthExisting({
  provider,
  bindingTicket,
}: {
  provider: Extract<IdentityProvider, 'github' | 'google'>;
  bindingTicket: string;
}) {
  const res = await axios.post(endpoints.auth.oauthBindExisting(provider), {
    binding_ticket: trimCredential(bindingTicket),
  });
  await setSession(requireAuthSession(res.data));
}

export async function walletNonce({ provider, address, chainId }: WalletNonceParams) {
  const res = await axios.post(endpoints.auth.walletNonce, {
    provider,
    address: trimCredential(address),
    chain_id: chainId,
  });
  return requireApiData<{ message: string; nonce: string }>(res.data);
}

export async function walletSignIn({
  provider,
  address,
  chainId,
  message,
  signature,
}: WalletSignInParams) {
  const res = await axios.post(endpoints.auth.walletSignIn, {
    provider,
    address: trimCredential(address),
    chain_id: chainId,
    message,
    signature,
  });
  return requireApiData<WalletSignInResponse>(res.data);
}

export async function requestWalletEmailCode({
  walletTicket,
  email,
  lang,
}: {
  walletTicket: string;
  email: string;
  lang: string;
}) {
  await axios.post(endpoints.auth.walletEmailCode, {
    wallet_ticket: trimCredential(walletTicket),
    email: trimCredential(email),
    lang: trimCredential(lang),
  });
}

export async function completeWallet({
  walletTicket,
  email,
  emailVerificationCode,
}: {
  walletTicket: string;
  email: string;
  emailVerificationCode: string;
}) {
  const res = await axios.post(endpoints.auth.walletComplete, {
    wallet_ticket: trimCredential(walletTicket),
    email: trimCredential(email),
    email_verification_code: trimCredential(emailVerificationCode),
  });
  await setSession(requireAuthSession(res.data));
}

export async function applyAuthenticatedSession(response: AuthSessionResponse) {
  await setSession(assertAuthSession(response));
}

/** **************************************
 * Password reset
 *************************************** */
export const requestPasswordReset = async ({
  email,
  lang,
  resetOrigin,
}: PasswordResetRequestParams): Promise<void> => {
  await axios.post(endpoints.auth.passwordResetRequest, {
    email: trimCredential(email),
    lang: trimCredential(lang),
    reset_origin: trimCredential(resetOrigin),
  });
};

export const confirmPasswordReset = async ({
  token,
  password,
}: PasswordResetConfirmParams): Promise<void> => {
  await axios.post(endpoints.auth.passwordResetConfirm, {
    token: trimCredential(token),
    password: trimCredential(password),
  });
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
  return assertAuthSession(session);
}

function assertAuthSession(session: AuthSessionResponse): AuthSessionResponse {
  if (!session.access_token || !session.refresh_token) {
    throw new Error('Auth tokens not found in response');
  }

  return session;
}
