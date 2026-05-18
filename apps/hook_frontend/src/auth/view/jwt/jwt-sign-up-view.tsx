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
import { useRouter } from 'src/routes/hooks';
import { RouterLink } from 'src/routes/components';

import { useCaptchaConfig } from 'src/actions/captcha';
import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { signUp } from '../../context/jwt';
import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { AuthCaptcha } from '../../components/cap-widget';
import { signUpSchema } from '../../context/jwt/validation';
import { SignUpTerms } from '../../components/sign-up-terms';

// ----------------------------------------------------------------------

type SignUpSchemaType = z.infer<ReturnType<typeof signUpSchema>>;

// ----------------------------------------------------------------------

export function JwtSignUpView() {
  const router = useRouter();
  const { t } = useTranslate('auth');

  const showPassword = useBoolean();

  const { checkUserSession } = useAuthContext();
  const captchaConfig = useCaptchaConfig();

  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);

  const captchaEnabled = captchaConfig.data?.registration_captcha_enabled ?? false;
  const captchaUnavailable = captchaConfig.isLoading || !!captchaConfig.error;
  const visibleErrorMessage =
    errorMessage ?? (captchaConfig.error ? getErrorMessage(captchaConfig.error) : null);

  const defaultValues: SignUpSchemaType = {
    username: '',
    email: '',
    password: '',
  };
  const schema = useMemo(() => signUpSchema(t), [t]);

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
      await signUp({
        username: data.username,
        email: data.email,
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
        name="username"
        label={t('fields.username')}
        placeholder={t('placeholders.username')}
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Field.Text
        name="email"
        label={t('fields.email')}
        placeholder={t('placeholders.email')}
        slotProps={{ inputLabel: { shrink: true } }}
      />

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
                  <Iconify icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />

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
          captchaConfig.isLoading ? t('common.loading', { ns: 'common' }) : t('actions.signUpLoading')
        }
      >
        {t('actions.signUp')}
      </Button>
    </Box>
  );

  return (
    <>
      <FormHead
        title={t('signUp.title')}
        description={
          <>
            {t('signUp.hasAccount')}{' '}
            <Link component={RouterLink} href={paths.auth.jwt.signIn} variant="subtitle2">
              {t('signUp.signIn')}
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

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>

      <SignUpTerms />
    </>
  );
}
