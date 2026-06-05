'use client';

import type { IdentityProvider } from 'src/types/rbac';
import type { IconifyProps } from 'src/components/iconify';
import type { AuthConfig, WalletProviderPublicConfig } from 'src/actions/auth-config';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { WalletConnectControl } from 'src/auth/components/wallet-connect-control';

type Props = {
  providers?: AuthConfig['providers'];
  loading: IdentityProvider | null;
  onOAuth: (provider: Extract<IdentityProvider, 'github' | 'google'>) => void;
  onWallet?: (
    provider: Extract<IdentityProvider, 'evm'>,
    config: WalletProviderPublicConfig
  ) => void;
};

export function JwtSocialSignIn(props: Props) {
  const { providers } = props;
  const hasOAuth = providers?.github.enabled || providers?.google.enabled;
  const hasWallet = providers?.evm.enabled && Boolean(props.onWallet);

  if (!hasOAuth && !hasWallet) {
    return null;
  }

  return (
    <Box sx={{ mb: 3 }}>
      <ProviderButtons {...props} />
    </Box>
  );
}

function ProviderButtons({ providers, loading, onOAuth, onWallet }: Props) {
  const { t } = useTranslate('auth');

  return (
    <>
      <Divider sx={{ mb: 2 }}>{t('social.or')}</Divider>
      <Stack spacing={1.5}>
        {providers?.github.enabled && (
          <ProviderButton
            label={t('social.github')}
            icon="socials:github"
            loading={loading === 'github'}
            onClick={() => onOAuth('github')}
          />
        )}
        {providers?.google.enabled && (
          <ProviderButton
            label={t('social.google')}
            icon="socials:google"
            loading={loading === 'google'}
            onClick={() => onOAuth('google')}
          />
        )}
        {providers?.evm.enabled && onWallet && (
          <WalletProviderButton
            label={t('social.evm')}
            connectedSignInLabel={t('social.evmConnectedSignIn')}
            loading={loading === 'evm'}
            onSignIn={() => onWallet?.('evm', providers.evm)}
          />
        )}
      </Stack>
    </>
  );
}

function ProviderButton({
  icon,
  label,
  loading,
  onClick,
}: {
  icon: IconifyProps['icon'];
  label: string;
  loading: boolean;
  onClick: VoidFunction;
}) {
  return (
    <Button
      fullWidth
      color="inherit"
      variant="outlined"
      loading={loading}
      startIcon={<Iconify icon={icon} />}
      onClick={onClick}
    >
      {label}
    </Button>
  );
}

function WalletProviderButton({
  label,
  connectedSignInLabel,
  loading,
  onSignIn,
}: {
  label: string;
  connectedSignInLabel: string;
  loading: boolean;
  onSignIn: VoidFunction;
}) {
  return (
    <WalletConnectControl
      label={label}
      connectedActionLabel={connectedSignInLabel}
      loading={loading}
      onAction={onSignIn}
    />
  );
}
