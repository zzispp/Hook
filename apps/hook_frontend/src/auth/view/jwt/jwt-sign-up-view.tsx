'use client';

import * as z from 'zod';
import { useState } from 'react';
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

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { signUp } from '../../context/jwt';
import { useAuthContext } from '../../hooks';
import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { AuthCaptcha } from '../../components/cap-widget';
import { SignUpTerms } from '../../components/sign-up-terms';
import { emailSchema, passwordSchema, usernameSchema } from '../../context/jwt/validation';

// ----------------------------------------------------------------------

export type SignUpSchemaType = z.infer<typeof SignUpSchema>;

export const SignUpSchema = z.object({
  username: usernameSchema,
  email: emailSchema,
  password: passwordSchema,
});

// ----------------------------------------------------------------------

export function JwtSignUpView() {
  const router = useRouter();

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

  const methods = useForm({
    resolver: zodResolver(SignUpSchema),
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
      setErrorMessage('Please complete CAPTCHA verification');
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
        label="Username"
        placeholder="username"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Field.Text
        name="email"
        label="Email address"
        placeholder="name@example.com"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Field.Text
        name="password"
        label="Password"
        placeholder="8+ characters"
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
        loadingIndicator={captchaConfig.isLoading ? 'Loading...' : 'Create account...'}
      >
        Create account
      </Button>
    </Box>
  );

  return (
    <>
      <FormHead
        title="Get started absolutely free"
        description={
          <>
            {`Already have an account? `}
            <Link component={RouterLink} href={paths.auth.jwt.signIn} variant="subtitle2">
              Get started
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
