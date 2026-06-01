'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { IdentityProvider } from 'src/types/rbac';

import { useRef, useState, useEffect, useCallback } from 'react';

import { paths } from 'src/routes/paths';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import { completeAccountOAuthCallback } from 'src/actions/account';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import {
  bindOAuthExisting,
  completeOAuthCallback,
  applyAuthenticatedSession,
} from '../../context/jwt';

export const ACCOUNT_OAUTH_BINDING_KEY = 'hook.account.oauth.binding';

export type OAuthProvider = Extract<IdentityProvider, 'github' | 'google'>;

export type BindingState = {
  bindingTicket: string;
  provider: OAuthProvider;
  email: string;
  username: string;
};

export type OAuthCallbackContentProps = {
  binding: BindingState | null;
  errorMessage: string | null;
  loading: boolean;
  submitting: boolean;
  onCancelBinding: () => void;
  onConfirmBinding: () => void;
};

type CallbackParams = {
  code: string;
  state: string;
};

type CallbackState = {
  binding: BindingState | null;
  errorMessage: string | null;
  loading: boolean;
  submitting: boolean;
  setBinding: Dispatch<SetStateAction<BindingState | null>>;
  setErrorMessage: Dispatch<SetStateAction<string | null>>;
  setLoading: Dispatch<SetStateAction<boolean>>;
  setSubmitting: Dispatch<SetStateAction<boolean>>;
};

type OAuthCallbackFlowOptions = {
  provider: OAuthProvider;
  accountBinding: boolean;
  authLoading: boolean;
  authenticated: boolean;
  missingParamsMessage: string;
  searchParams: ReturnType<typeof useSearchParams>;
  router: ReturnType<typeof useRouter>;
  finishAuthenticated: () => Promise<void>;
  setBinding: Dispatch<SetStateAction<BindingState | null>>;
  setErrorMessage: Dispatch<SetStateAction<string | null>>;
  setLoading: Dispatch<SetStateAction<boolean>>;
};

export function useOAuthCallbackController(provider: OAuthProvider): OAuthCallbackContentProps {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t } = useTranslate('auth');
  const { authenticated, loading: authLoading, checkUserSession } = useAuthContext();
  const accountBinding = accountOAuthBindingProvider() === provider;
  const state = useOAuthCallbackState();
  const finishAuthenticated = useCallback(async () => {
    await checkUserSession?.();
    router.replace(paths.dashboard.root);
  }, [checkUserSession, router]);
  const { cancelBinding, confirmBinding } = useOAuthCallbackActions({
    binding: state.binding,
    finishAuthenticated,
    router,
    setErrorMessage: state.setErrorMessage,
    setSubmitting: state.setSubmitting,
  });

  useOAuthCallbackFlow({
    provider,
    accountBinding,
    authLoading,
    authenticated,
    searchParams,
    router,
    finishAuthenticated,
    missingParamsMessage: t('social.oauthMissingParams'),
    setBinding: state.setBinding,
    setErrorMessage: state.setErrorMessage,
    setLoading: state.setLoading,
  });

  return {
    binding: state.binding,
    errorMessage: state.errorMessage,
    loading: state.loading,
    submitting: state.submitting,
    onCancelBinding: cancelBinding,
    onConfirmBinding: confirmBinding,
  };
}

function useOAuthCallbackState(): CallbackState {
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [binding, setBinding] = useState<BindingState | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  return {
    binding,
    errorMessage,
    loading,
    submitting,
    setBinding,
    setErrorMessage,
    setLoading,
    setSubmitting,
  };
}

function useOAuthCallbackActions({
  binding,
  finishAuthenticated,
  router,
  setErrorMessage,
  setSubmitting,
}: {
  binding: BindingState | null;
  finishAuthenticated: () => Promise<void>;
  router: ReturnType<typeof useRouter>;
  setErrorMessage: Dispatch<SetStateAction<string | null>>;
  setSubmitting: Dispatch<SetStateAction<boolean>>;
}) {
  const confirmBinding = useCallback(async () => {
    if (!binding) return;
    setSubmitting(true);
    setErrorMessage(null);
    try {
      await bindOAuthExisting({
        provider: binding.provider,
        bindingTicket: binding.bindingTicket,
      });
      await finishAuthenticated();
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }, [binding, finishAuthenticated, setErrorMessage, setSubmitting]);

  const cancelBinding = useCallback(() => {
    router.replace(paths.auth.jwt.signIn);
  }, [router]);

  return { cancelBinding, confirmBinding };
}

function useOAuthCallbackFlow(options: OAuthCallbackFlowOptions) {
  const handledRef = useRef(false);

  useEffect(() => {
    if (options.authLoading || handledRef.current) return;
    const params = oauthCallbackParams(options.searchParams);
    handledRef.current = true;
    runOAuthCallbackFlow(options, params);
  }, [options]);
}

function runOAuthCallbackFlow(options: OAuthCallbackFlowOptions, params: CallbackParams | null) {
  if (!params) {
    options.setErrorMessage(options.missingParamsMessage);
    options.setLoading(false);
    return;
  }
  if (options.accountBinding) {
    startAccountBindingCallback(options, params);
    return;
  }
  if (options.authenticated) {
    redirectAuthenticatedCallback(options.router);
    return;
  }
  startSignInCallback(options, params);
}

function startAccountBindingCallback(options: OAuthCallbackFlowOptions, params: CallbackParams) {
  void completeAccountBindingCallback(options, params);
}

function redirectAuthenticatedCallback(router: ReturnType<typeof useRouter>) {
  router.replace(paths.dashboard.root);
}

function startSignInCallback(options: OAuthCallbackFlowOptions, params: CallbackParams) {
  void completeSignInCallback(options, params);
}

async function completeAccountBindingCallback(
  options: Pick<OAuthCallbackFlowOptions, 'provider' | 'router' | 'setErrorMessage' | 'setLoading'>,
  params: CallbackParams
) {
  try {
    await completeAccountOAuthCallback({ provider: options.provider, ...params });
    options.router.replace(`${paths.dashboard.profile}?provider_linked=1`);
  } catch (error) {
    options.setErrorMessage(getErrorMessage(error));
  } finally {
    window.sessionStorage.removeItem(ACCOUNT_OAUTH_BINDING_KEY);
    options.setLoading(false);
  }
}

async function completeSignInCallback(
  options: Pick<
    OAuthCallbackFlowOptions,
    'finishAuthenticated' | 'provider' | 'setBinding' | 'setErrorMessage' | 'setLoading'
  >,
  params: CallbackParams
) {
  try {
    const result = await completeOAuthCallback({ provider: options.provider, ...params });
    if (result.status === 'authenticated') {
      await applyAuthenticatedSession(result);
      await options.finishAuthenticated();
      return;
    }
    options.setBinding({
      bindingTicket: result.binding_ticket,
      provider: result.provider as OAuthProvider,
      email: result.email,
      username: result.username,
    });
  } catch (error) {
    options.setErrorMessage(getErrorMessage(error));
  } finally {
    options.setLoading(false);
  }
}

function oauthCallbackParams(searchParams: ReturnType<typeof useSearchParams>) {
  const code = searchParams.get('code')?.trim();
  const state = searchParams.get('state')?.trim();
  return code && state ? { code, state } : null;
}

function accountOAuthBindingProvider() {
  if (typeof window === 'undefined') return null;
  return window.sessionStorage.getItem(ACCOUNT_OAUTH_BINDING_KEY);
}
