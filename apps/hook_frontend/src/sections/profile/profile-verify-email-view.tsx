'use client';

import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';
import { useRouter } from 'src/routes/hooks';
import { RouterLink } from 'src/routes/components';

import { SentIcon } from 'src/assets/icons';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAccountProfile, verifyAccountEmail, requestAccountPasswordEmailCode } from 'src/actions/account';

import { toast } from 'src/components/snackbar';
import { Form } from 'src/components/hook-form';
import { Iconify } from 'src/components/iconify';

import { FormHead } from 'src/auth/components/form-head';

import { ProfileEmailCodeFields } from './profile-email-code-fields';
import {
  profileEmailCodeSchema,
  profileEmailCodeDefaultValues,
  type ProfileEmailCodeSchemaType,
} from './profile-email-code-schema';

export function ProfileVerifyEmailView() {
  const { t } = useTranslate('common');
  const profile = useAccountProfile();

  return (
    <DashboardContent maxWidth="sm">
      {profile.error ? <Alert severity="error">{profile.error.message}</Alert> : null}

      {profile.data && !profile.data.system && profile.emailCodeAvailable ? (
        <ProfileVerifyEmailForm email={profile.data.email} />
      ) : null}
      {profile.data && !profile.data.system && !profile.emailCodeAvailable ? (
        <Alert severity="warning">{t('profile.messages.emailConfigUnavailable')}</Alert>
      ) : null}
      {profile.data?.system ? <Alert severity="warning">{t('profile.systemPasswordBlocked')}</Alert> : null}
    </DashboardContent>
  );
}

function ProfileVerifyEmailForm({ email }: { email: string }) {
  const { t } = useTranslate('common');
  const form = useProfileVerifyEmailForm(email);

  return (
    <>
      <FormHead
        icon={<SentIcon />}
        title={t('profile.verifyEmailTitle')}
        description={t('profile.verifyEmailDescription')}
      />

      <Form methods={form.methods} onSubmit={form.onSubmit}>
        <ProfileEmailCodeFields
          sendingCode={form.sendingCode}
          isSubmitting={form.methods.formState.isSubmitting}
          onSendCode={form.sendCode}
        />

        <Button
          fullWidth
          size="large"
          type="submit"
          variant="contained"
          loading={form.methods.formState.isSubmitting}
          loadingIndicator={t('profile.verifyEmailLoading')}
          sx={{ mt: 3 }}
        >
          {t('profile.verifyEmail')}
        </Button>
      </Form>

      <ReturnToProfileLink />
    </>
  );
}

function useProfileVerifyEmailForm(email: string) {
  const router = useRouter();
  const { t, currentLang } = useTranslate('common');
  const [sendingCode, setSendingCode] = useState(false);
  const schema = useMemo(() => profileEmailCodeSchema(t), [t]);
  const methods = useForm<ProfileEmailCodeSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: profileEmailCodeDefaultValues(email),
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
      await verifyAccountEmail({ emailVerificationCode: data.code });
      toast.success(t('profile.messages.emailVerified'));
      router.replace(paths.dashboard.profile);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    }
  });

  return { methods, onSubmit, sendingCode, sendCode };
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
