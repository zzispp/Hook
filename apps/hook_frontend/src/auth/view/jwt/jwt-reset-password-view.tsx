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

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { confirmPasswordReset } from '../../context/jwt';
import { resetPasswordSchema } from '../../context/jwt/validation';

type ResetPasswordSchemaType = z.infer<ReturnType<typeof resetPasswordSchema>>;

const PASSWORD_RESET_SUCCESS_QUERY = 'reset=success';

export function JwtResetPasswordView() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const showPassword = useBoolean();
  const { t } = useTranslate('auth');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const token = searchParams.get('token')?.trim() ?? '';
  const schema = useMemo(() => resetPasswordSchema(t), [t]);

  const methods = useForm<ResetPasswordSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: { password: '' },
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    setErrorMessage(null);
    if (!token) {
      setErrorMessage(t('resetPassword.missingToken'));
      return;
    }
    try {
      await confirmPasswordReset({ token, password: data.password });
      router.replace(`${paths.auth.jwt.signIn}?${PASSWORD_RESET_SUCCESS_QUERY}`);
    } catch (error) {
      console.error(error);
      setErrorMessage(getErrorMessage(error));
    }
  });

  return (
    <>
      <FormHead
        title={t('resetPassword.title')}
        description={t('resetPassword.description')}
        sx={{ textAlign: { xs: 'center', md: 'left' } }}
      />

      {!token && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {t('resetPassword.missingToken')}
        </Alert>
      )}

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {errorMessage}
        </Alert>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
          <Field.Text
            name="password"
            label={t('fields.newPassword')}
            placeholder={t('placeholders.password')}
            type={showPassword.value ? 'text' : 'password'}
            disabled={!token}
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

          <Button
            fullWidth
            color="inherit"
            size="large"
            type="submit"
            variant="contained"
            disabled={!token}
            loading={isSubmitting}
            loadingIndicator={t('actions.resetPasswordLoading')}
          >
            {t('actions.resetPassword')}
          </Button>
        </Box>
      </Form>

      <Box sx={{ mt: 3, textAlign: 'center' }}>
        <Link component={RouterLink} href={paths.auth.jwt.signIn} variant="subtitle2">
          {t('actions.backToSignIn')}
        </Link>
      </Box>
    </>
  );
}
