'use client';

import type * as z from 'zod';

import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { PasswordIcon } from 'src/assets/icons';
import { useTranslate } from 'src/locales/use-locales';

import { Form, Field } from 'src/components/hook-form';

import { getErrorMessage } from '../../utils';
import { FormHead } from '../../components/form-head';
import { requestPasswordReset } from '../../context/jwt';
import { forgotPasswordSchema } from '../../context/jwt/validation';

type ForgotPasswordSchemaType = z.infer<ReturnType<typeof forgotPasswordSchema>>;

export function JwtForgotPasswordView() {
  const { t, currentLang } = useTranslate('auth');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const schema = useMemo(() => forgotPasswordSchema(t), [t]);

  const methods = useForm<ForgotPasswordSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: { email: '' },
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    setErrorMessage(null);
    setSuccessMessage(null);
    try {
      await requestPasswordReset({
        email: data.email,
        lang: currentLang.value,
        resetOrigin: window.location.origin,
      });
      setSuccessMessage(t('forgotPassword.success'));
    } catch (error) {
      console.error(error);
      setErrorMessage(getErrorMessage(error));
    }
  });

  return (
    <>
      <FormHead
        icon={<PasswordIcon />}
        title={t('forgotPassword.title')}
        description={t('forgotPassword.description')}
      />

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {errorMessage}
        </Alert>
      )}

      {!!successMessage && (
        <Alert severity="success" sx={{ mb: 3 }}>
          {successMessage}
        </Alert>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
          <Field.Text
            autoFocus
            name="email"
            label={t('fields.email')}
            placeholder={t('placeholders.email')}
            slotProps={{ inputLabel: { shrink: true } }}
          />

          <Button
            fullWidth
            size="large"
            type="submit"
            variant="contained"
            loading={isSubmitting}
            loadingIndicator={t('actions.requestPasswordResetLoading')}
          >
            {t('actions.requestPasswordReset')}
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
