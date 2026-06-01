'use client';

import type * as z from 'zod';
import type { IdentityProvider } from 'src/types/rbac';
import type { WalletSignInResponse } from '../../context/jwt';
import type { WalletProviderPublicConfig } from 'src/actions/auth-config';

import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useCaptchaConfig } from 'src/actions/captcha';
import { useTranslate } from 'src/locales/use-locales';
import { useAuthConfig } from 'src/actions/auth-config';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { JwtSocialSignIn } from './jwt-social-sign-in';
import { signInSchema } from '../../context/jwt/validation';
import { useWalletSigning } from '../../context/jwt/wallet-signing';
import { JwtPasswordSignInForm } from './jwt-password-sign-in-form';
import {
  startOAuth,
  walletNonce,
  walletSignIn,
  signInWithPassword,
  applyAuthenticatedSession,
} from '../../context/jwt';

// ----------------------------------------------------------------------

type SignInSchemaType = z.infer<ReturnType<typeof signInSchema>>;

// ----------------------------------------------------------------------

const PASSWORD_RESET_SUCCESS_PARAM = 'reset';
const PASSWORD_RESET_SUCCESS_VALUE = 'success';

// ----------------------------------------------------------------------

export function JwtSignInView() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t } = useTranslate('auth');

  const { checkUserSession } = useAuthContext();
  const { signWalletMessage, connectWalletAccount } = useWalletSigning();
  const authConfig = useAuthConfig();
  const captchaConfig = useCaptchaConfig();

  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [socialLoading, setSocialLoading] = useState<IdentityProvider | null>(null);
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);

  const providers = authConfig.data?.providers;
  const captchaEnabled = captchaConfig.data?.login_captcha_enabled ?? false;
  const captchaUnavailable = captchaConfig.isLoading || !!captchaConfig.error;
  const visibleErrorMessage =
    errorMessage ?? (captchaConfig.error ? getErrorMessage(captchaConfig.error) : null);
  const passwordResetSucceeded =
    searchParams.get(PASSWORD_RESET_SUCCESS_PARAM) === PASSWORD_RESET_SUCCESS_VALUE;

  const defaultValues: SignInSchemaType = {
    identifier: '',
    password: '',
  };
  const schema = useMemo(() => signInSchema(t), [t]);

  const methods = useForm({
    resolver: zodResolver(schema),
    defaultValues,
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    setErrorMessage(null);
    if (captchaConfig.error) {
      setErrorMessage(getErrorMessage(captchaConfig.error));
      return;
    }
    if (captchaConfig.isLoading) {
      return;
    }
    if (captchaEnabled && !captchaToken) {
      setErrorMessage(t('captcha.required'));
      return;
    }

    try {
      await signInWithPassword({
        identifier: data.identifier,
        password: data.password,
        captchaToken: captchaEnabled ? (captchaToken ?? undefined) : undefined,
      });
      await checkUserSession?.();

      router.refresh();
    } catch (error) {
      console.error(error);
      const feedbackMessage = getErrorMessage(error);
      setErrorMessage(feedbackMessage);
      if (captchaEnabled) {
        setCaptchaToken(null);
        setCaptchaResetKey((value) => value + 1);
      }
    }
  });

  return (
    <>
      <FormHead
        title={t('signIn.title')}
        description={
          <>
            {t('signIn.noAccount')}{' '}
            <Link component={RouterLink} href={paths.auth.jwt.signUp} variant="subtitle2">
              {t('signIn.createAccount')}
            </Link>
          </>
        }
        sx={{ textAlign: { xs: 'center', md: 'left' } }}
      />

      {!!visibleErrorMessage && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {visibleErrorMessage}
        </Alert>
      )}

      {passwordResetSucceeded && (
        <Alert severity="success" sx={{ mb: 3 }}>
          {t('resetPassword.success')}
        </Alert>
      )}

      <JwtSocialSignIn
        providers={providers}
        loading={socialLoading}
        onOAuth={(provider) =>
          runSocialAction(setErrorMessage, setSocialLoading, provider, async () => {
            const { authorization_url } = await startOAuth(provider);
            window.location.assign(authorization_url);
          })
        }
        onWallet={(provider, config) =>
          runSocialAction(setErrorMessage, setSocialLoading, provider, async () => {
            const account = await connectWalletAccount(walletScope(provider, config));
            const challenge = await walletNonce(account);
            const signed = await signWalletMessage({
              ...account,
              message: challenge.message,
            });
            const result = await walletSignIn({
              ...account,
              message: challenge.message,
              signature: signed.signature,
            });
            await handleWalletResult(result, checkUserSession, router.refresh, t);
          })
        }
      />

      <JwtPasswordSignInForm
        methods={methods}
        onSubmit={onSubmit}
        captchaEnabled={captchaEnabled}
        captchaResetKey={captchaResetKey}
        captchaUnavailable={captchaUnavailable}
        loading={isSubmitting}
        captchaLoading={captchaConfig.isLoading}
        signInLabel={t('actions.signIn')}
        signInLoadingLabel={t('actions.signInLoading')}
        captchaLoadingLabel={t('common.loading', { ns: 'common' })}
        identifierLabel={t('fields.identifier')}
        identifierPlaceholder={t('placeholders.identifier')}
        passwordLabel={t('fields.password')}
        passwordPlaceholder={t('placeholders.password')}
        forgotPasswordLabel={t('signIn.forgotPassword')}
        onCaptchaTokenChange={setCaptchaToken}
      />
    </>
  );
}

async function runSocialAction(
  setErrorMessage: (message: string | null) => void,
  setSocialLoading: (provider: IdentityProvider | null) => void,
  provider: IdentityProvider,
  action: () => Promise<void>
) {
  setErrorMessage(null);
  setSocialLoading(provider);
  try {
    await action();
  } catch (error) {
    console.error(error);
    setErrorMessage(getErrorMessage(error));
  } finally {
    setSocialLoading(null);
  }
}

function walletScope(
  provider: Extract<IdentityProvider, 'evm'>,
  config: WalletProviderPublicConfig
) {
  return {
    provider,
    chainId: config.evm_chain_ids[0],
  };
}

async function handleWalletResult(
  result: WalletSignInResponse,
  checkUserSession: (() => Promise<void>) | undefined,
  refresh: VoidFunction,
  t: ReturnType<typeof useTranslate>['t']
) {
  if (result.status === 'authenticated') {
    await applyAuthenticatedSession(result);
    await checkUserSession?.();
    refresh();
    return;
  }

  throw new Error(t('social.walletAccountRequired', { address: result.address }));
}
