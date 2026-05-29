'use client';

import type * as z from 'zod';
import type { UseFormReturn } from 'react-hook-form';
import type { UseBooleanReturn } from 'minimal-shared/hooks';
import type { signInSchema } from '../../context/jwt/validation';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { AuthCaptcha } from '../../components/cap-widget';

type SignInSchemaType = z.infer<ReturnType<typeof signInSchema>>;

type Props = {
  methods: UseFormReturn<SignInSchemaType>;
  onSubmit: () => void;
  captchaEnabled: boolean;
  captchaResetKey: number;
  captchaUnavailable: boolean;
  loading: boolean;
  captchaLoading: boolean;
  signInLabel: string;
  signInLoadingLabel: string;
  captchaLoadingLabel: string;
  identifierLabel: string;
  identifierPlaceholder: string;
  passwordLabel: string;
  passwordPlaceholder: string;
  forgotPasswordLabel: string;
  onCaptchaTokenChange: (token: string | null) => void;
};

export function JwtPasswordSignInForm({
  methods,
  onSubmit,
  captchaEnabled,
  captchaResetKey,
  captchaUnavailable,
  loading,
  captchaLoading,
  signInLabel,
  signInLoadingLabel,
  captchaLoadingLabel,
  identifierLabel,
  identifierPlaceholder,
  passwordLabel,
  passwordPlaceholder,
  forgotPasswordLabel,
  onCaptchaTokenChange,
}: Props) {
  const showPassword = useBoolean();

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
        <Field.Text
          name="identifier"
          label={identifierLabel}
          placeholder={identifierPlaceholder}
          slotProps={{ inputLabel: { shrink: true } }}
        />
        <PasswordField
          showPassword={showPassword}
          label={passwordLabel}
          placeholder={passwordPlaceholder}
          forgotPasswordLabel={forgotPasswordLabel}
        />
        <AuthCaptcha
          enabled={captchaEnabled}
          resetKey={captchaResetKey}
          onTokenChange={onCaptchaTokenChange}
        />
        <Button
          fullWidth
          color="inherit"
          size="large"
          type="submit"
          variant="contained"
          disabled={captchaUnavailable}
          loading={loading || captchaLoading}
          loadingIndicator={captchaLoading ? captchaLoadingLabel : signInLoadingLabel}
        >
          {signInLabel}
        </Button>
      </Box>
    </Form>
  );
}

function PasswordField({
  showPassword,
  label,
  placeholder,
  forgotPasswordLabel,
}: {
  showPassword: UseBooleanReturn;
  label: string;
  placeholder: string;
  forgotPasswordLabel: string;
}) {
  return (
    <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
      <Link
        component={RouterLink}
        href={paths.auth.jwt.forgotPassword}
        variant="body2"
        color="inherit"
        sx={{ alignSelf: 'flex-end' }}
      >
        {forgotPasswordLabel}
      </Link>
      <Field.Text
        name="password"
        label={label}
        placeholder={placeholder}
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
    </Box>
  );
}
