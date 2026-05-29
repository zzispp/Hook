'use client';

import type { UserIdentitySummary } from 'src/types/rbac';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  useAccountProfile,
  deleteAccountIdentity,
  changeAccountPassword,
  requestAccountPasswordEmailCode,
} from 'src/actions/account';

import { Label } from 'src/components/label';
import { toast } from 'src/components/snackbar';

import { providerColor, providerLabel } from './provider-utils';

export function ProfileView() {
  const { t, currentLang } = useTranslate('common');
  const profile = useAccountProfile();
  const [code, setCode] = useState('');
  const [password, setPassword] = useState('');
  const [sendingCode, setSendingCode] = useState(false);
  const [changingPassword, setChangingPassword] = useState(false);
  const [unlinkingId, setUnlinkingId] = useState<string | null>(null);

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

  const changePassword = async () => {
    setChangingPassword(true);
    try {
      await changeAccountPassword({ emailVerificationCode: code, password });
      setCode('');
      setPassword('');
      await profile.refresh();
      toast.success(t('profile.messages.passwordChanged'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    } finally {
      setChangingPassword(false);
    }
  };

  const unlinkIdentity = async (identity: UserIdentitySummary) => {
    setUnlinkingId(identity.id);
    try {
      await deleteAccountIdentity(identity.id);
      await profile.refresh();
      toast.success(t('profile.messages.providerUnlinked'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    } finally {
      setUnlinkingId(null);
    }
  };

  const user = profile.data;

  return (
    <DashboardContent maxWidth="md">
      <Typography variant="h4" sx={{ mb: 3 }}>
        {t('profile.title')}
      </Typography>

      {profile.error && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {profile.error.message}
        </Alert>
      )}

      <Stack spacing={3}>
        <Card>
          <CardHeader title={t('profile.accountTitle')} />
          <CardContent>
            <Stack spacing={2}>
              <ReadOnlyRow label={t('profile.username')} value={user?.username ?? ''} />
              <ReadOnlyRow label={t('profile.email')} value={user?.email ?? ''} />
              <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
                <Label color={user?.email_verified ? 'success' : 'warning'} variant="soft">
                  {user?.email_verified
                    ? t('profile.emailVerified')
                    : t('profile.emailUnverified')}
                </Label>
                <Label color={user?.password_set ? 'success' : 'warning'} variant="soft">
                  {user?.password_set
                    ? t('profile.passwordSet')
                    : t('profile.passwordNotSet')}
                </Label>
              </Stack>
            </Stack>
          </CardContent>
        </Card>

        <Card>
          <CardHeader title={t('profile.passwordTitle')} />
          <CardContent>
            <Stack spacing={2}>
              <Button
                color="inherit"
                variant="outlined"
                loading={sendingCode}
                onClick={sendCode}
                sx={{ alignSelf: 'flex-start' }}
              >
                {t('profile.sendCode')}
              </Button>
              <TextField
                fullWidth
                label={t('profile.emailCode')}
                value={code}
                onChange={(event) => setCode(event.target.value)}
              />
              <TextField
                fullWidth
                type="password"
                label={t('profile.newPassword')}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
              <Button
                color="inherit"
                variant="contained"
                loading={changingPassword}
                onClick={changePassword}
                sx={{ alignSelf: 'flex-start' }}
              >
                {t('profile.changePassword')}
              </Button>
            </Stack>
          </CardContent>
        </Card>

        <Card>
          <CardHeader title={t('profile.providersTitle')} />
          <CardContent>
            <Stack spacing={2}>
              {(user?.identities ?? []).length === 0 ? (
                <Typography variant="body2" color="text.secondary">
                  {t('profile.noProviders')}
                </Typography>
              ) : (
                user?.identities.map((identity) => (
                  <ProviderRow
                    key={identity.id}
                    identity={identity}
                    loading={unlinkingId === identity.id}
                    onUnlink={() => unlinkIdentity(identity)}
                  />
                ))
              )}
            </Stack>
          </CardContent>
        </Card>
      </Stack>
    </DashboardContent>
  );
}

function ReadOnlyRow({ label, value }: { label: string; value: string }) {
  return (
    <Box>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2">{value || '-'}</Typography>
    </Box>
  );
}

function ProviderRow({
  identity,
  loading,
  onUnlink,
}: {
  identity: UserIdentitySummary;
  loading: boolean;
  onUnlink: VoidFunction;
}) {
  const { t } = useTranslate('common');

  return (
    <Box>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} alignItems={{ sm: 'center' }}>
        <Chip
          size="small"
          label={providerLabel(identity.provider)}
          color={providerColor(identity.provider)}
          variant="soft"
        />
        <Box sx={{ minWidth: 0, flex: 1 }}>
          <Typography variant="body2" noWrap>
            {identity.display_name || identity.email || identity.provider_subject}
          </Typography>
          <Typography variant="caption" color="text.secondary" noWrap>
            {identity.provider_subject}
          </Typography>
        </Box>
        <Button color="error" variant="text" loading={loading} onClick={onUnlink}>
          {t('profile.unlink')}
        </Button>
      </Stack>
      <Divider sx={{ mt: 2 }} />
    </Box>
  );
}
