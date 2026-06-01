'use client';

import type { AuthConfig } from 'src/actions/auth-config';
import type { SystemUser, UserIdentitySummary } from 'src/types/rbac';
import type { OAuthProvider, WalletProvider } from './profile-provider-card';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';

import { paths } from 'src/routes/paths';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import { useAuthConfig } from 'src/actions/auth-config';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  useAccountProfile,
  linkAccountWallet,
  startAccountOAuth,
  deleteAccountIdentity,
} from 'src/actions/account';

import { Label } from 'src/components/label';
import { toast } from 'src/components/snackbar';

import { walletNonce } from 'src/auth/context/jwt';
import { useWalletSigning } from 'src/auth/context/jwt/wallet-signing';
import { ACCOUNT_OAUTH_BINDING_KEY } from 'src/auth/view/jwt/jwt-oauth-callback-view';

import { ProviderCard } from './profile-provider-card';

const PROVIDER_LINKED_PARAM = 'provider_linked';

export function ProfileView() {
  const { t } = useTranslate('common');
  const router = useRouter();
  const searchParams = useSearchParams();
  const profile = useAccountProfile();
  const authConfig = useAuthConfig();
  const walletSigning = useWalletSigning();
  const identityActions = useProfileIdentityActions(() => profile.refresh(), router.refresh);
  const providerActions = useProviderLinkActions(authConfig.data, walletSigning, () =>
    profile.refresh()
  );

  const user = profile.data;
  const canUseSelfService = user !== undefined && !user.system;

  useEffect(() => {
    if (searchParams.get(PROVIDER_LINKED_PARAM) !== '1') return;
    toast.success(t('profile.messages.providerLinked'));
    router.replace(paths.dashboard.profile);
  }, [router, searchParams, t]);

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
        <AccountCard user={user} emailCodeAvailable={profile.emailCodeAvailable} />

        {canUseSelfService && (
          <>
            <PasswordSummaryCard
              passwordSet={user.password_set}
              emailCodeAvailable={profile.emailCodeAvailable}
            />
            <ProviderCard
              identities={user.identities}
              authConfig={authConfig.data}
              linkingProvider={providerActions.linkingProvider}
              unlinkingId={identityActions.unlinkingId}
              unlinkTarget={identityActions.unlinkTarget}
              onOAuthLink={providerActions.linkOAuth}
              onWalletLink={providerActions.linkWallet}
              onCancelUnlink={identityActions.cancelUnlink}
              onUnlink={identityActions.unlinkIdentity}
              onConfirmUnlink={identityActions.confirmUnlink}
            />
          </>
        )}
      </Stack>
    </DashboardContent>
  );
}

function PasswordSummaryCard({
  passwordSet,
  emailCodeAvailable,
}: {
  passwordSet: boolean;
  emailCodeAvailable: boolean;
}) {
  const { t } = useTranslate('common');
  const router = useRouter();

  const openChangePassword = () => {
    if (!emailCodeAvailable && !passwordSet) {
      toast.error(t('profile.messages.passwordResetRequiresAdmin'));
      return;
    }
    router.push(paths.dashboard.profileChangePassword);
  };

  return (
    <Card>
      <CardHeader title={t('profile.passwordTitle')} />
      <CardContent>
        <Stack spacing={2} alignItems="flex-start">
          <Label color={passwordSet ? 'success' : 'warning'} variant="soft">
            {passwordSet ? t('profile.passwordSet') : t('profile.passwordNotSet')}
          </Label>
          <Typography variant="body2" color="text.secondary">
            {t('profile.passwordDescription')}
          </Typography>
          <Button variant="contained" onClick={openChangePassword}>
            {t('profile.changePassword')}
          </Button>
        </Stack>
      </CardContent>
    </Card>
  );
}

function useProfileIdentityActions(refresh: () => Promise<unknown>, refreshRouter: VoidFunction) {
  const { t } = useTranslate('common');
  const [unlinkingId, setUnlinkingId] = useState<string | null>(null);
  const [unlinkTarget, setUnlinkTarget] = useState<UserIdentitySummary | null>(null);

  const unlinkIdentity = async (identity: UserIdentitySummary) => {
    setUnlinkTarget(identity);
  };

  const confirmUnlink = async () => {
    if (!unlinkTarget) return;
    setUnlinkingId(unlinkTarget.id);
    try {
      await deleteAccountIdentity(unlinkTarget.id);
      await refresh();
      refreshRouter();
      toast.success(t('profile.messages.providerUnlinked'));
      setUnlinkTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    } finally {
      setUnlinkingId(null);
    }
  };

  const cancelUnlink = () => {
    if (!unlinkingId) setUnlinkTarget(null);
  };

  return { unlinkingId, unlinkTarget, unlinkIdentity, confirmUnlink, cancelUnlink };
}

function useProviderLinkActions(
  authConfig: AuthConfig | undefined,
  walletSigning: ReturnType<typeof useWalletSigning>,
  refresh: () => Promise<unknown>
) {
  const { t } = useTranslate('common');
  const [linkingProvider, setLinkingProvider] = useState<UserIdentitySummary['provider'] | null>(
    null
  );

  const linkOAuth = async (provider: OAuthProvider) => {
    setLinkingProvider(provider);
    try {
      const { authorization_url } = await startAccountOAuth(provider);
      window.sessionStorage.setItem(ACCOUNT_OAUTH_BINDING_KEY, provider);
      window.location.assign(authorization_url);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
      setLinkingProvider(null);
    }
  };

  const linkWallet = async (provider: WalletProvider) => {
    const config = authConfig?.providers.evm;
    if (!config?.enabled || config.evm_chain_ids.length === 0) {
      toast.error(t('profile.messages.saveFailed'));
      return;
    }
    setLinkingProvider(provider);
    try {
      const account = await walletSigning.connectWalletAccount({
        provider,
        chainId: config.evm_chain_ids[0],
      });
      const challenge = await walletNonce(account);
      const signed = await walletSigning.signWalletMessage({
        ...account,
        message: challenge.message,
      });
      await linkAccountWallet({
        ...account,
        message: challenge.message,
        signature: signed.signature,
      });
      await refresh();
      toast.success(t('profile.messages.providerLinked'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('profile.messages.saveFailed'));
    } finally {
      setLinkingProvider(null);
    }
  };

  return { linkingProvider, linkOAuth, linkWallet };
}

function AccountCard({
  user,
  emailCodeAvailable,
}: {
  user?: SystemUser;
  emailCodeAvailable: boolean;
}) {
  const { t } = useTranslate('common');
  const router = useRouter();

  const openVerifyEmail = () => {
    if (!emailCodeAvailable) {
      toast.error(t('profile.messages.emailConfigUnavailable'));
      return;
    }
    router.push(paths.dashboard.profileVerifyEmail);
  };

  return (
    <Card>
      <CardHeader title={t('profile.accountTitle')} />
      <CardContent>
        <Stack spacing={2}>
          <ReadOnlyRow label={t('profile.username')} value={user?.username ?? ''} />
          <ReadOnlyRow label={t('profile.email')} value={user?.email ?? ''} />
          <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
            <Label color={user?.email_verified ? 'success' : 'warning'} variant="soft">
              {user?.email_verified ? t('profile.emailVerified') : t('profile.emailUnverified')}
            </Label>
            <Label color={user?.password_set ? 'success' : 'warning'} variant="soft">
              {user?.password_set ? t('profile.passwordSet') : t('profile.passwordNotSet')}
            </Label>
          </Stack>
          {user && !user.email_verified ? (
            <Button
              variant="outlined"
              sx={{ alignSelf: 'flex-start' }}
              onClick={openVerifyEmail}
            >
              {t('profile.verifyEmail')}
            </Button>
          ) : null}
        </Stack>
      </CardContent>
    </Card>
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
