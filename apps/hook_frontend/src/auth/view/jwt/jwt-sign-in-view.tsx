'use client';

import type * as z from 'zod';

import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useCaptchaConfig } from 'src/actions/captcha';
import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { signInWithPassword } from '../../context/jwt';
import { AuthCaptcha } from '../../components/cap-widget';
import { signInSchema } from '../../context/jwt/validation';

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

  const showPassword = useBoolean();

  const { checkUserSession } = useAuthContext();
  const captchaConfig = useCaptchaConfig();

  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);

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

  const renderForm = () => (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="identifier"
        label={t('fields.identifier')}
        placeholder={t('placeholders.identifier')}
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
        <Link
          component={RouterLink}
          href={paths.auth.jwt.forgotPassword}
          variant="body2"
          color="inherit"
          sx={{ alignSelf: 'flex-end' }}
        >
          {t('signIn.forgotPassword')}
        </Link>

        <Field.Text
          name="password"
          label={t('fields.password')}
          placeholder={t('placeholders.password')}
          type={showPassword.value ? 'text' : 'password'}
          slotProps={{
            inputLabel: { shrink: true },
            input: {
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={showPassword.onToggle} edge="end">
                    <Iconify
                      icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'}
                    />
                  </IconButton>
                </InputAdornment>
              ),
            },
          }}
        />
      </Box>

      <AuthCaptcha
        enabled={captchaEnabled}
        resetKey={captchaResetKey}
        onTokenChange={setCaptchaToken}
      />

      <Button
        fullWidth
        color="inherit"
        size="large"
        type="submit"
        variant="contained"
        disabled={captchaUnavailable}
        loading={isSubmitting || captchaConfig.isLoading}
        loadingIndicator={
          captchaConfig.isLoading ? t('common.loading', { ns: 'common' }) : t('actions.signInLoading')
        }
      >
        {t('actions.signIn')}
      </Button>
    </Box>
  );

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

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>
    </>
  );
}
