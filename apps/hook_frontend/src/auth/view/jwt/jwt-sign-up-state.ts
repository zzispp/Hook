'use client';

import type * as z from 'zod';
import type { SubmitHandler, UseFormReturn } from 'react-hook-form';
import type { IdentityProvider } from 'src/types/rbac';
import type { WalletProviderPublicConfig } from 'src/actions/auth-config';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo, useState, useEffect, useCallback } from 'react';

import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import { useCaptchaConfig } from 'src/actions/captcha';
import { useAuthConfig } from 'src/actions/auth-config';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { signUpSchema } from '../../context/jwt/validation';
import { useWalletSigning } from '../../context/jwt/wallet-signing';
import { signUp, startOAuth, walletNonce, walletRegister, requestRegistrationEmailCode } from '../../context/jwt';

export type SignUpSchemaType = z.infer<ReturnType<typeof signUpSchema>>;

type Translate = ReturnType<typeof useTranslate>['t'];
type Feedback = { error: string | null; success: string | null };
type AuthConfigState = ReturnType<typeof useAuthConfig>;
type CaptchaConfigState = ReturnType<typeof useCaptchaConfig>;

const DEFAULT_SIGN_UP_VALUES: SignUpSchemaType = {
  username: '',
  email: '',
  password: '',
  emailVerificationCode: '',
};
const DEFAULT_WALLET_FORM = { username: '', email: '', emailVerificationCode: '' };
const EMAIL_CODE_COOLDOWN_SECONDS = 60;
const AFF_QUERY_PARAM = 'aff';

export function useJwtSignUpState() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t, currentLang } = useTranslate('auth');
  const { checkUserSession } = useAuthContext();
  const walletSigning = useWalletSigning();
  const authConfig = useAuthConfig();
  const captchaConfig = useCaptchaConfig();
  const [feedback, setFeedback] = useState<Feedback>({ error: null, success: null });
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [socialLoading, setSocialLoading] = useState<IdentityProvider | null>(null);
  const [walletRegistration, setWalletRegistration] = useState<VerifiedWallet | null>(null);
  const [walletForm, setWalletForm] = useState(DEFAULT_WALLET_FORM);
  const [walletSubmitting, setWalletSubmitting] = useState(false);
  const [walletEmailCodeLoading, setWalletEmailCodeLoading] = useState(false);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);
  const flags = signUpFlags(authConfig, captchaConfig);
  const schema = useMemo(
    () => signUpSchema(t, { emailVerificationRequired: flags.emailVerificationEnabled }),
    [flags.emailVerificationEnabled, t]
  );
  const methods = useForm<SignUpSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: DEFAULT_SIGN_UP_VALUES,
  });
  const emailCode = useRegistrationEmailCodeRequest({ methods, lang: currentLang.value, t, setFeedback });
  const resetCaptcha = useCaptchaReset(flags.captchaEnabled, setCaptchaToken, setCaptchaResetKey);
  const submit = useSignUpSubmit({
    affCode: searchParams.get(AFF_QUERY_PARAM) ?? undefined,
    authConfig,
    captchaConfig,
    captchaToken,
    checkUserSession,
    flags,
    resetCaptcha,
    routerRefresh: router.refresh,
    setFeedback,
    t,
  });

  return {
    methods,
    form: {
      ...flags,
      captchaResetKey,
      emailCodeCooldownSeconds: emailCode.cooldownSeconds,
      isRequestingEmailCode: emailCode.isRequesting,
      onCaptchaTokenChange: setCaptchaToken,
      onRequestEmailCode: emailCode.request,
    },
    visibleErrorMessage: feedback.error ?? firstErrorMessage(authConfig.error, captchaConfig.error),
    successMessage: feedback.success,
    providers: authConfig.data?.providers,
    socialLoading,
    walletRegistrationOpen: Boolean(walletRegistration),
    walletForm,
    walletSubmitting,
    walletEmailCodeLoading,
    onOAuth: (provider: Extract<IdentityProvider, 'github' | 'google'>) =>
      void runSignUpOAuth(provider, searchParams.get(AFF_QUERY_PARAM) ?? undefined, {
        setFeedback,
        setSocialLoading,
      }),
    onWallet: (provider: Extract<IdentityProvider, 'evm'>, config: WalletProviderPublicConfig) =>
      void beginWalletRegistration(provider, config, walletSigning, {
        setFeedback,
        setSocialLoading,
        setWalletRegistration,
      }),
    onWalletFormChange: setWalletForm,
    onWalletEmailCode: () =>
      void requestWalletEmailCode(walletForm.email, currentLang.value, {
        setFeedback,
        setLoading: setWalletEmailCodeLoading,
        t,
      }),
    onWalletRegister: () =>
      void submitWalletRegistration(walletRegistration, walletForm, searchParams.get(AFF_QUERY_PARAM) ?? undefined, {
        checkUserSession,
        routerRefresh: router.refresh,
        setFeedback,
        setSubmitting: setWalletSubmitting,
      }),
    onCloseWalletRegister: () => setWalletRegistration(null),
    onSubmit: methods.handleSubmit(submit),
  };
}

type VerifiedWallet = {
  provider: Extract<IdentityProvider, 'evm'>;
  address: string;
  chainId?: number;
  message: string;
  signature: string;
};

async function beginWalletRegistration(
  provider: Extract<IdentityProvider, 'evm'>,
  config: WalletProviderPublicConfig,
  walletSigning: ReturnType<typeof useWalletSigning>,
  options: WalletBeginOptions
) {
  options.setFeedback(emptyFeedback());
  options.setSocialLoading(provider);
  try {
    const account = await walletSigning.connectWalletAccount({ provider, chainId: config.evm_chain_ids[0] });
    const challenge = await walletNonce(account);
    const signed = await walletSigning.signWalletMessage({ ...account, message: challenge.message });
    options.setWalletRegistration({ ...account, message: challenge.message, signature: signed.signature });
  } catch (error) {
    console.error(error);
    options.setFeedback({ error: getErrorMessage(error), success: null });
  } finally {
    options.setSocialLoading(null);
  }
}

type WalletBeginOptions = {
  setFeedback: (feedback: Feedback) => void;
  setSocialLoading: (provider: IdentityProvider | null) => void;
  setWalletRegistration: (wallet: VerifiedWallet) => void;
};

async function runSignUpOAuth(
  provider: Extract<IdentityProvider, 'github' | 'google'>,
  affCode: string | undefined,
  options: {
    setFeedback: (feedback: Feedback) => void;
    setSocialLoading: (provider: IdentityProvider | null) => void;
  }
) {
  options.setFeedback(emptyFeedback());
  options.setSocialLoading(provider);
  try {
    const { authorization_url } = await startOAuth(provider, affCode);
    window.location.assign(authorization_url);
  } catch (error) {
    console.error(error);
    options.setFeedback({ error: getErrorMessage(error), success: null });
  } finally {
    options.setSocialLoading(null);
  }
}

async function requestWalletEmailCode(
  email: string,
  lang: string,
  options: {
    setFeedback: (feedback: Feedback) => void;
    setLoading: (loading: boolean) => void;
    t: Translate;
  }
) {
  options.setFeedback(emptyFeedback());
  options.setLoading(true);
  try {
    await requestRegistrationEmailCode({ email, lang });
    options.setFeedback({ error: null, success: options.t('signUp.emailCodeSent') });
  } catch (error) {
    console.error(error);
    options.setFeedback({ error: getErrorMessage(error), success: null });
  } finally {
    options.setLoading(false);
  }
}

async function submitWalletRegistration(
  wallet: VerifiedWallet | null,
  form: typeof DEFAULT_WALLET_FORM,
  affCode: string | undefined,
  options: {
    checkUserSession?: () => Promise<void>;
    routerRefresh: () => void;
    setFeedback: (feedback: Feedback) => void;
    setSubmitting: (submitting: boolean) => void;
  }
) {
  if (!wallet) {
    return;
  }
  options.setFeedback(emptyFeedback());
  options.setSubmitting(true);
  try {
    await walletRegister({ ...wallet, ...form, affCode });
    await options.checkUserSession?.();
    options.routerRefresh();
  } catch (error) {
    console.error(error);
    options.setFeedback({ error: getErrorMessage(error), success: null });
  } finally {
    options.setSubmitting(false);
  }
}

function useRegistrationEmailCodeRequest(options: RegistrationEmailCodeRequestOptions) {
  const [isRequesting, setIsRequesting] = useState(false);
  const [cooldownSeconds, setCooldownSeconds] = useState(0);
  useEffect(() => {
    if (cooldownSeconds <= 0) {
      return undefined;
    }
    const timer = window.setTimeout(() => {
      setCooldownSeconds((value) => Math.max(0, value - 1));
    }, 1000);
    return () => window.clearTimeout(timer);
  }, [cooldownSeconds]);
  const request = useCallback(async () => {
    if (cooldownSeconds > 0) {
      return;
    }
    options.setFeedback(emptyFeedback());
    const emailValid = await options.methods.trigger('email', { shouldFocus: true });

    if (!emailValid) {
      return;
    }

    setIsRequesting(true);
    try {
      await requestRegistrationEmailCode({
        email: options.methods.getValues('email'),
        lang: options.lang,
      });
      setCooldownSeconds(EMAIL_CODE_COOLDOWN_SECONDS);
      options.setFeedback({ error: null, success: options.t('signUp.emailCodeSent') });
    } catch (error) {
      console.error(error);
      options.setFeedback({ error: getErrorMessage(error), success: null });
    } finally {
      setIsRequesting(false);
    }
  }, [cooldownSeconds, options]);

  return { cooldownSeconds, isRequesting, request };
}

type RegistrationEmailCodeRequestOptions = {
  methods: UseFormReturn<SignUpSchemaType>;
  lang: string;
  t: Translate;
  setFeedback: (feedback: Feedback) => void;
};

function useCaptchaReset(
  captchaEnabled: boolean,
  setCaptchaToken: (token: string | null) => void,
  setCaptchaResetKey: React.Dispatch<React.SetStateAction<number>>
) {
  return useCallback(() => {
    if (!captchaEnabled) {
      return;
    }
    setCaptchaToken(null);
    setCaptchaResetKey((value) => value + 1);
  }, [captchaEnabled, setCaptchaResetKey, setCaptchaToken]);
}

function useSignUpSubmit(options: SignUpSubmitOptions): SubmitHandler<SignUpSchemaType> {
  return useCallback(
    async (data) => {
      options.setFeedback(emptyFeedback());
      const configError = firstErrorMessage(options.authConfig.error, options.captchaConfig.error);

      if (configError) {
        options.setFeedback({ error: configError, success: null });
        return;
      }
      if (options.flags.configLoading) {
        return;
      }
      if (options.flags.captchaEnabled && !options.captchaToken) {
        options.setFeedback({ error: options.t('captcha.required'), success: null });
        return;
      }

      await submitSignUp(data, options);
    },
    [options]
  );
}

type SignUpSubmitOptions = {
  affCode?: string;
  authConfig: AuthConfigState;
  captchaConfig: CaptchaConfigState;
  captchaToken: string | null;
  checkUserSession?: () => Promise<void>;
  flags: SignUpFlags;
  resetCaptcha: () => void;
  routerRefresh: () => void;
  setFeedback: (feedback: Feedback) => void;
  t: Translate;
};

async function submitSignUp(data: SignUpSchemaType, options: SignUpSubmitOptions) {
  try {
    await signUp({
      username: data.username,
      email: data.email,
      password: data.password,
      emailVerificationCode: options.flags.emailVerificationEnabled
        ? data.emailVerificationCode
        : undefined,
      captchaToken: options.flags.captchaEnabled ? (options.captchaToken ?? undefined) : undefined,
      affCode: options.affCode,
    });
    await options.checkUserSession?.();
    options.routerRefresh();
  } catch (error) {
    console.error(error);
    options.setFeedback({ error: getErrorMessage(error), success: null });
    options.resetCaptcha();
  }
}

type SignUpFlags = {
  emailVerificationEnabled: boolean;
  captchaEnabled: boolean;
  configLoading: boolean;
  formUnavailable: boolean;
};

function signUpFlags(authConfig: AuthConfigState, captchaConfig: CaptchaConfigState): SignUpFlags {
  const configLoading = authConfig.isLoading || captchaConfig.isLoading;

  return {
    emailVerificationEnabled:
      authConfig.data?.registration_email_verification_enabled ?? false,
    captchaEnabled: captchaConfig.data?.registration_captcha_enabled ?? false,
    configLoading,
    formUnavailable: configLoading || !!authConfig.error || !!captchaConfig.error,
  };
}

function firstErrorMessage(...errors: unknown[]) {
  const error = errors.find(Boolean);

  return error ? getErrorMessage(error) : null;
}

function emptyFeedback(): Feedback {
  return { error: null, success: null };
}
