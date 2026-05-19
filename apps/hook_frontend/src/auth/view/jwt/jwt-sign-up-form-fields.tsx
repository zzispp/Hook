'use client';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Field } from 'src/components/hook-form';

import { AuthCaptcha } from '../../components/cap-widget';

type JwtSignUpFormFieldsProps = {
  emailVerificationEnabled: boolean;
  captchaEnabled: boolean;
  captchaResetKey: number;
  emailCodeCooldownSeconds: number;
  formUnavailable: boolean;
  configLoading: boolean;
  isSubmitting: boolean;
  isRequestingEmailCode: boolean;
  onCaptchaTokenChange: (token: string | null) => void;
  onRequestEmailCode: () => void;
};

type EmailCodeProps = Pick<
  JwtSignUpFormFieldsProps,
  | 'emailCodeCooldownSeconds'
  | 'formUnavailable'
  | 'isSubmitting'
  | 'isRequestingEmailCode'
  | 'onRequestEmailCode'
>;

type SubmitProps = Pick<JwtSignUpFormFieldsProps, 'formUnavailable' | 'isSubmitting'> & {
  configLoading: boolean;
};

export function JwtSignUpFormFields(props: JwtSignUpFormFieldsProps) {
  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <UsernameField />
      <EmailField />
      {props.emailVerificationEnabled && <EmailCodeField {...props} />}
      <PasswordField />
      <AuthCaptcha
        enabled={props.captchaEnabled}
        resetKey={props.captchaResetKey}
        onTokenChange={props.onCaptchaTokenChange}
      />
      <SubmitButton
        formUnavailable={props.formUnavailable}
        isSubmitting={props.isSubmitting}
        configLoading={props.configLoading}
      />
    </Box>
  );
}

function UsernameField() {
  const { t } = useTranslate('auth');

  return (
    <Field.Text
      name="username"
      label={t('fields.username')}
      placeholder={t('placeholders.username')}
      slotProps={{ inputLabel: { shrink: true } }}
    />
  );
}

function EmailField() {
  const { t } = useTranslate('auth');

  return (
    <Field.Text
      name="email"
      label={t('fields.email')}
      placeholder={t('placeholders.email')}
      slotProps={{ inputLabel: { shrink: true } }}
    />
  );
}

function EmailCodeField({
  emailCodeCooldownSeconds,
  formUnavailable,
  isSubmitting,
  isRequestingEmailCode,
  onRequestEmailCode,
}: EmailCodeProps) {
  const { t } = useTranslate('auth');
  const actionText =
    emailCodeCooldownSeconds > 0
      ? `${t('actions.sendRegistrationCode')} (${emailCodeCooldownSeconds}s)`
      : t('actions.sendRegistrationCode');

  return (
    <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
      <Box component="label" sx={{ typography: 'subtitle2', color: 'text.secondary' }}>
        {t('fields.emailVerificationCode')}
      </Box>
      <Field.Code
        name="emailVerificationCode"
        autoFocus={false}
        slotProps={{
          textField: {
            inputProps: {
              inputMode: 'numeric',
              'aria-label': t('placeholders.emailVerificationCode'),
            },
          },
        }}
      />
      <Button
        fullWidth
        color="inherit"
        type="button"
        variant="outlined"
        loading={isRequestingEmailCode}
        disabled={formUnavailable || isSubmitting || emailCodeCooldownSeconds > 0}
        loadingIndicator={t('actions.sendRegistrationCodeLoading')}
        onClick={onRequestEmailCode}
      >
        {actionText}
      </Button>
    </Box>
  );
}

function PasswordField() {
  const { t } = useTranslate('auth');
  const showPassword = useBoolean();

  return (
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
  );
}

function SubmitButton({ formUnavailable, isSubmitting, configLoading }: SubmitProps) {
  const { t } = useTranslate('auth');

  return (
    <Button
      fullWidth
      color="inherit"
      size="large"
      type="submit"
      variant="contained"
      disabled={formUnavailable}
      loading={isSubmitting || configLoading}
      loadingIndicator={
        configLoading ? t('common.loading', { ns: 'common' }) : t('actions.signUpLoading')
      }
    >
      {t('actions.signUp')}
    </Button>
  );
}
