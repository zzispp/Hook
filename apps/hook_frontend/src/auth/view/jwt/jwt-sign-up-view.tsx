'use client';

import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { useTranslate } from 'src/locales/use-locales';

import { Form } from 'src/components/hook-form';

import { FormHead } from '../../components/form-head';
import { JwtSocialSignIn } from './jwt-social-sign-in';
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
    providers,
    socialLoading,
    walletRegistrationOpen,
    walletForm,
    walletSubmitting,
    walletEmailCodeLoading,
    onOAuth,
    onWallet,
    onWalletFormChange,
    onWalletEmailCode,
    onWalletRegister,
    onCloseWalletRegister,
    onSubmit,
    form: {
      emailVerificationEnabled,
      captchaEnabled,
      captchaResetKey,
      emailCodeCooldownSeconds,
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

      <JwtSocialSignIn
        providers={providers}
        loading={socialLoading}
        onOAuth={onOAuth}
        onWallet={onWallet}
      />

      <Form methods={methods} onSubmit={onSubmit}>
        <JwtSignUpFormFields
          emailVerificationEnabled={emailVerificationEnabled}
          captchaEnabled={captchaEnabled}
          captchaResetKey={captchaResetKey}
          emailCodeCooldownSeconds={emailCodeCooldownSeconds}
          formUnavailable={formUnavailable}
          configLoading={configLoading}
          isSubmitting={isSubmitting}
          isRequestingEmailCode={isRequestingEmailCode}
          onCaptchaTokenChange={onCaptchaTokenChange}
          onRequestEmailCode={onRequestEmailCode}
        />
      </Form>

      <SignUpTerms />

      <WalletRegisterDialog
        open={walletRegistrationOpen}
        form={walletForm}
        submitting={walletSubmitting}
        emailCodeLoading={walletEmailCodeLoading}
        onChange={onWalletFormChange}
        onRequestEmailCode={onWalletEmailCode}
        onSubmit={onWalletRegister}
        onClose={onCloseWalletRegister}
      />
    </>
  );
}

function WalletRegisterDialog({
  open,
  form,
  submitting,
  emailCodeLoading,
  onChange,
  onRequestEmailCode,
  onSubmit,
  onClose,
}: {
  open: boolean;
  form: { username: string; email: string; emailVerificationCode: string };
  submitting: boolean;
  emailCodeLoading: boolean;
  onChange: (form: { username: string; email: string; emailVerificationCode: string }) => void;
  onRequestEmailCode: VoidFunction;
  onSubmit: VoidFunction;
  onClose: VoidFunction;
}) {
  const { t } = useTranslate('auth');

  return (
    <Dialog fullWidth maxWidth="xs" open={open} onClose={onClose}>
      <DialogTitle>{t('walletRegister.title')}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <TextField
            fullWidth
            label={t('fields.username')}
            value={form.username}
            onChange={(event) => onChange({ ...form, username: event.target.value })}
          />
          <TextField
            fullWidth
            label={t('fields.email')}
            value={form.email}
            onChange={(event) => onChange({ ...form, email: event.target.value })}
          />
          <TextField
            fullWidth
            label={t('fields.emailVerificationCode')}
            value={form.emailVerificationCode}
            onChange={(event) => onChange({ ...form, emailVerificationCode: event.target.value })}
          />
          <Button variant="outlined" loading={emailCodeLoading} onClick={onRequestEmailCode}>
            {t('actions.sendRegistrationCode')}
          </Button>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button color="inherit" onClick={onClose}>
          {t('common.cancel', { ns: 'common' })}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('walletRegister.submit')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
