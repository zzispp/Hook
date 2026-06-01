'use client';

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

import { SentIcon } from 'src/assets/icons';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAccountProfile, changeAccountPassword, requestAccountPasswordEmailCode } from 'src/actions/account';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

import { FormHead } from 'src/auth/components/form-head';
import { PASSWORD_MIN_LENGTH } from 'src/auth/context/jwt/validation';

import { ProfileEmailCodeFields } from './profile-email-code-fields';
import {
  profilePasswordSchema,
  profilePasswordDefaultValues,
  type ProfilePasswordSchemaType,
  profilePasswordWithCurrentSchema,
  profilePasswordWithCurrentDefaultValues,
  type ProfilePasswordWithCurrentSchemaType,
} from './profile-password-schema';

export function ProfilePasswordView() {
  const { t } = useTranslate('common');
  const profile = useAccountProfile();

  return (
    <DashboardContent maxWidth="sm">
      {profile.error ? <Alert severity="error">{profile.error.message}</Alert> : null}

      {profile.data && !profile.data.system ? (
        <ProfilePasswordForm
          email={profile.data.email}
          emailCodeAvailable={profile.emailCodeAvailable}
          passwordSet={profile.data.password_set}
        />
      ) : null}
      {profile.data?.system ? <Alert severity="warning">{t('profile.systemPasswordBlocked')}</Alert> : null}
    </DashboardContent>
  );
}

function ProfilePasswordForm({
  email,
  emailCodeAvailable,
  passwordSet,
}: {
  email: string;
  emailCodeAvailable: boolean;
  passwordSet: boolean;
}) {
  const { t } = useTranslate('common');
  const description = emailCodeAvailable
    ? t('profile.changePasswordDescription')
    : t('profile.changePasswordWithCurrentDescription');

  if (!emailCodeAvailable && !passwordSet) {
    return <Alert severity="warning">{t('profile.messages.passwordResetRequiresAdmin')}</Alert>;
  }

  return (
    <>
      <FormHead
        icon={<SentIcon />}
        title={t('profile.changePasswordTitle')}
        description={description}
      />

      {emailCodeAvailable ? <EmailCodePasswordForm email={email} /> : <CurrentPasswordForm />}

      <ReturnToProfileLink />
    </>
  );
}

function EmailCodePasswordForm({ email }: { email: string }) {
  const form = useEmailCodePasswordForm(email);

  return (
    <Form methods={form.methods} onSubmit={form.onSubmit}>
      <ProfilePasswordFields
        identitySlot={
          <ProfileEmailCodeFields
            sendingCode={form.sendingCode}
            isSubmitting={form.methods.formState.isSubmitting}
            onSendCode={form.sendCode}
          />
        }
        isSubmitting={form.methods.formState.isSubmitting}
      />
    </Form>
  );
}

function useEmailCodePasswordForm(email: string) {
  const router = useRouter();
  const { t, currentLang } = useTranslate('common');
  const [sendingCode, setSendingCode] = useState(false);
  const schema = useMemo(() => profilePasswordSchema(t), [t]);
  const methods = useForm<ProfilePasswordSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: profilePasswordDefaultValues(email),
  });

  const sendCode = async () => {
    setSendingCode(true);
    try {
      await requestAccountPasswordEmailCode(currentLang.value);
      toast.success(t('profile.messages.codeSent'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    } finally {
      setSendingCode(false);
    }
  };

  const onSubmit = methods.handleSubmit(async (data) => {
    try {
      await changeAccountPassword({ emailVerificationCode: data.code, password: data.password });
      toast.success(t('profile.messages.passwordChanged'));
      router.replace(paths.dashboard.profile);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    }
  });

  return { methods, onSubmit, sendingCode, sendCode };
}

function CurrentPasswordForm() {
  const form = useCurrentPasswordForm();
  const { t } = useTranslate('common');

  return (
    <Form methods={form.methods} onSubmit={form.onSubmit}>
      <ProfilePasswordFields
        identitySlot={
          <PasswordField
            name="currentPassword"
            label={t('profile.currentPassword')}
            showPassword={form.showPassword}
          />
        }
        isSubmitting={form.methods.formState.isSubmitting}
      />
    </Form>
  );
}

function useCurrentPasswordForm() {
  const router = useRouter();
  const { t } = useTranslate('common');
  const showPassword = useBoolean();
  const schema = useMemo(() => profilePasswordWithCurrentSchema(t), [t]);
  const methods = useForm<ProfilePasswordWithCurrentSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: profilePasswordWithCurrentDefaultValues(),
  });

  const onSubmit = methods.handleSubmit(async (data) => {
    try {
      await changeAccountPassword({
        currentPassword: data.currentPassword,
        password: data.password,
      });
      toast.success(t('profile.messages.passwordChanged'));
      router.replace(paths.dashboard.profile);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    }
  });

  return { methods, onSubmit, showPassword };
}

function ProfilePasswordFields({
  isSubmitting,
  identitySlot,
}: {
  isSubmitting: boolean;
  identitySlot: React.ReactNode;
}) {
  const { t } = useTranslate('common');
  const showPassword = useBoolean();

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      {identitySlot}

      <PasswordField name="password" label={t('profile.newPassword')} showPassword={showPassword} />

      <PasswordField
        name="confirmPassword"
        label={t('profile.confirmPassword')}
        showPassword={showPassword}
      />

      <Button
        fullWidth
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator={t('profile.changePasswordLoading')}
      >
        {t('profile.changePassword')}
      </Button>
    </Box>
  );
}

function PasswordField({
  name,
  label,
  showPassword,
}: {
  name: 'currentPassword' | 'password' | 'confirmPassword';
  label: string;
  showPassword: ReturnType<typeof useBoolean>;
}) {
  const { t } = useTranslate('common');

  return (
    <Field.Text
      name={name}
      label={label}
      placeholder={t('profile.passwordPlaceholder', { min: PASSWORD_MIN_LENGTH })}
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

function ReturnToProfileLink() {
  const { t } = useTranslate('common');

  return (
    <Link
      component={RouterLink}
      href={paths.dashboard.profile}
      color="inherit"
      variant="subtitle2"
      sx={{
        gap: 0.5,
        mx: 'auto',
        mt: 3,
        display: 'flex',
        alignItems: 'center',
        width: 'fit-content',
      }}
    >
      <Iconify width={16} icon="eva:arrow-ios-back-fill" />
      {t('profile.backToProfile')}
    </Link>
  );
}
