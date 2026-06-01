'use client';

import type { AuthConfig } from 'src/actions/auth-config';
import type { UserIdentitySummary } from 'src/types/rbac';
import type { IconifyProps } from 'src/components/iconify';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { providerColor, providerLabel } from './provider-utils';

export type OAuthProvider = Extract<UserIdentitySummary['provider'], 'github' | 'google'>;
export type WalletProvider = Extract<UserIdentitySummary['provider'], 'evm'>;

type ProviderCardProps = {
  identities: UserIdentitySummary[];
  authConfig?: AuthConfig;
  linkingProvider: UserIdentitySummary['provider'] | null;
  unlinkingId: string | null;
  unlinkTarget: UserIdentitySummary | null;
  onOAuthLink: (provider: OAuthProvider) => void;
  onWalletLink: (provider: WalletProvider) => void;
  onCancelUnlink: VoidFunction;
  onUnlink: (identity: UserIdentitySummary) => void;
  onConfirmUnlink: VoidFunction;
};

export function ProviderCard(props: ProviderCardProps) {
  const { t } = useTranslate('common');
  const linkedProviders = new Set(props.identities.map((identity) => identity.provider));

  return (
    <Card>
      <CardHeader title={t('profile.providersTitle')} />
      <CardContent>
        <Stack spacing={2}>
          <ProviderLinkButtons
            authConfig={props.authConfig}
            linkedProviders={linkedProviders}
            linkingProvider={props.linkingProvider}
            onOAuthLink={props.onOAuthLink}
            onWalletLink={props.onWalletLink}
          />
          <ProviderList
            identities={props.identities}
            unlinkingId={props.unlinkingId}
            onUnlink={props.onUnlink}
          />
        </Stack>
      </CardContent>
      <ConfirmDialog
        open={!!props.unlinkTarget}
        onClose={props.onCancelUnlink}
        title={t('profile.unlinkConfirmTitle')}
        content={t('profile.unlinkConfirmContent', {
          provider: props.unlinkTarget ? providerLabel(props.unlinkTarget.provider) : '',
        })}
        cancelText={t('profile.cancel')}
        action={
          <Button
            color="error"
            variant="contained"
            loading={!!props.unlinkTarget && props.unlinkingId === props.unlinkTarget.id}
            onClick={props.onConfirmUnlink}
          >
            {t('profile.unlink')}
          </Button>
        }
      />
    </Card>
  );
}

function ProviderLinkButtons({
  authConfig,
  linkedProviders,
  linkingProvider,
  onOAuthLink,
  onWalletLink,
}: {
  authConfig?: AuthConfig;
  linkedProviders: Set<UserIdentitySummary['provider']>;
  linkingProvider: UserIdentitySummary['provider'] | null;
  onOAuthLink: (provider: OAuthProvider) => void;
  onWalletLink: (provider: WalletProvider) => void;
}) {
  const { t } = useTranslate('common');
  const enabledProviders = (['github', 'google', 'evm'] as const).filter((provider) =>
    providerEnabled(authConfig, provider)
  );

  if (enabledProviders.length === 0) return null;

  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1}>
      {enabledProviders.map((provider) => {
        const linked = linkedProviders.has(provider);
        return (
          <Button
            key={provider}
            color="inherit"
            variant="outlined"
            loading={linkingProvider === provider}
            disabled={linked}
            startIcon={<Iconify icon={providerIcon(provider)} />}
            onClick={() => (provider === 'evm' ? onWalletLink(provider) : onOAuthLink(provider))}
          >
            {linked
              ? t('profile.providerLinked', { provider: providerLabel(provider) })
              : t('profile.linkProvider', { provider: providerLabel(provider) })}
          </Button>
        );
      })}
    </Stack>
  );
}

function ProviderList({
  identities,
  unlinkingId,
  onUnlink,
}: Pick<ProviderCardProps, 'identities' | 'unlinkingId' | 'onUnlink'>) {
  const { t } = useTranslate('common');

  if (identities.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('profile.noProviders')}
      </Typography>
    );
  }

  return identities.map((identity) => (
    <ProviderRow
      key={identity.id}
      identity={identity}
      loading={unlinkingId === identity.id}
      onUnlink={() => onUnlink(identity)}
    />
  ));
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

function providerEnabled(authConfig: AuthConfig | undefined, provider: UserIdentitySummary['provider']) {
  if (!authConfig) return false;
  return authConfig.providers[provider].enabled;
}

function providerIcon(provider: UserIdentitySummary['provider']) {
  const icons: Record<UserIdentitySummary['provider'], IconifyProps['icon']> = {
    github: 'socials:github',
    google: 'socials:google',
    evm: 'solar:wad-of-money-bold',
  };
  return icons[provider];
}
