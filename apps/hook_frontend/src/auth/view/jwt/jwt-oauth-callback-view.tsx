'use client';

import type { IdentityProvider } from 'src/types/rbac';

import { useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

import { paths } from 'src/routes/paths';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import { completeAccountOAuthCallback } from 'src/actions/account';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { bindOAuthExisting, completeOAuthCallback, applyAuthenticatedSession } from '../../context/jwt';

type Props = {
  provider: Extract<IdentityProvider, 'github' | 'google'>;
};

export const ACCOUNT_OAUTH_BINDING_KEY = 'hook.account.oauth.binding';

type BindingState = {
  bindingTicket: string;
  provider: Extract<IdentityProvider, 'github' | 'google'>;
  email: string;
  username: string;
};

export function JwtOAuthCallbackView({ provider }: Props) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t } = useTranslate('auth');
  const { checkUserSession } = useAuthContext();
  const accountBinding = accountOAuthBindingProvider() === provider;

  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [binding, setBinding] = useState<BindingState | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const finishAuthenticated = useCallback(async () => {
    await checkUserSession?.();
    router.replace(paths.dashboard.root);
  }, [checkUserSession, router]);

  useEffect(() => {
    const code = searchParams.get('code')?.trim();
    const state = searchParams.get('state')?.trim();
    if (!code || !state) {
      setErrorMessage(t('social.oauthMissingParams'));
      setLoading(false);
      return;
    }
    if (accountBinding) {
      completeAccountOAuthCallback({ provider, code, state })
        .then(() => {
          router.replace(`${paths.dashboard.profile}?provider_linked=1`);
        })
        .catch((error) => setErrorMessage(getErrorMessage(error)))
        .finally(() => {
          window.sessionStorage.removeItem(ACCOUNT_OAUTH_BINDING_KEY);
          setLoading(false);
        });
      return;
    }
    completeOAuthCallback({ provider, code, state })
      .then(async (result) => {
        if (result.status === 'authenticated') {
          await applyAuthenticatedSession(result);
          await finishAuthenticated();
          return;
        }
        setBinding({
          bindingTicket: result.binding_ticket,
          provider: result.provider as Extract<IdentityProvider, 'github' | 'google'>,
          email: result.email,
          username: result.username,
        });
      })
      .catch((error) => setErrorMessage(getErrorMessage(error)))
      .finally(() => setLoading(false));
  }, [accountBinding, finishAuthenticated, provider, router, searchParams, t]);

  const confirmBinding = async () => {
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
  };

  return (
    <Box sx={{ textAlign: 'center' }}>
      {loading && (
        <Box sx={{ py: 4 }}>
          <CircularProgress />
          <Typography variant="body2" sx={{ mt: 2, color: 'text.secondary' }}>
            {t('social.oauthProcessing')}
          </Typography>
        </Box>
      )}

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
          {errorMessage}
        </Alert>
      )}

      {binding && (
        <Box sx={{ gap: 2, display: 'flex', flexDirection: 'column' }}>
          <Typography variant="h5">{t('social.bindingRequiredTitle')}</Typography>
          <Typography variant="body2" color="text.secondary">
            {t('social.bindingRequiredDescription', {
              provider: providerLabel(binding.provider),
              email: binding.email,
              username: binding.username,
            })}
          </Typography>
          <Button color="inherit" variant="contained" loading={submitting} onClick={confirmBinding}>
            {t('social.confirmBinding')}
          </Button>
          <Button color="inherit" variant="text" onClick={() => router.replace(paths.auth.jwt.signIn)}>
            {t('social.cancelBinding')}
          </Button>
        </Box>
      )}
    </Box>
  );
}

function providerLabel(provider: Extract<IdentityProvider, 'github' | 'google'>) {
  return provider === 'github' ? 'GitHub' : 'Google';
}

function accountOAuthBindingProvider() {
  if (typeof window === 'undefined') return null;
  return window.sessionStorage.getItem(ACCOUNT_OAUTH_BINDING_KEY);
}
