'use client';

import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { useTranslate } from 'src/locales/use-locales';

import { Form } from 'src/components/hook-form';

import { FormHead } from '../../components/form-head';
import { useJwtSignUpState } from './jwt-sign-up-state';
import { SignUpTerms } from '../../components/sign-up-terms';
import { JwtSignUpFormFields } from './jwt-sign-up-form-fields';

export function JwtSignUpView() {
  const { t } = useTranslate('auth');
  const signUp = useJwtSignUpState();
  const {
    methods,
    visibleErrorMessage,
    successMessage,
    onSubmit,
    form: {
      emailVerificationEnabled,
      captchaEnabled,
      captchaResetKey,
      formUnavailable,
      configLoading,
      isRequestingEmailCode,
      onCaptchaTokenChange,
      onRequestEmailCode,
    },
  } = signUp;
  const isSubmitting = methods.formState.isSubmitting;

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

      {!!successMessage && (
        <Alert severity="success" sx={{ mb: 3 }}>
          {successMessage}
        </Alert>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        <JwtSignUpFormFields
          emailVerificationEnabled={emailVerificationEnabled}
          captchaEnabled={captchaEnabled}
          captchaResetKey={captchaResetKey}
          formUnavailable={formUnavailable}
          configLoading={configLoading}
          isSubmitting={isSubmitting}
          isRequestingEmailCode={isRequestingEmailCode}
          onCaptchaTokenChange={onCaptchaTokenChange}
          onRequestEmailCode={onRequestEmailCode}
        />
      </Form>

      <SignUpTerms />
    </>
  );
}
