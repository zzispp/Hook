'use client';

import type * as z from 'zod';
import type { SubmitHandler, UseFormReturn } from 'react-hook-form';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo, useState, useEffect, useCallback } from 'react';

import { useRouter } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import { useCaptchaConfig } from 'src/actions/captcha';
import { useAuthConfig } from 'src/actions/auth-config';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { signUpSchema } from '../../context/jwt/validation';
import { signUp, requestRegistrationEmailCode } from '../../context/jwt';

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
const EMAIL_CODE_COOLDOWN_SECONDS = 60;

export function useJwtSignUpState() {
  const router = useRouter();
  const { t, currentLang } = useTranslate('auth');
  const { checkUserSession } = useAuthContext();
  const authConfig = useAuthConfig();
  const captchaConfig = useCaptchaConfig();
  const [feedback, setFeedback] = useState<Feedback>({ error: null, success: null });
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
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
    onSubmit: methods.handleSubmit(submit),
  };
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
