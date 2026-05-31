'use client';

import type { IdentityProvider } from 'src/types/rbac';
import type { IconifyProps } from 'src/components/iconify';
import type { AuthConfig, WalletProviderPublicConfig } from 'src/actions/auth-config';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

export type WalletTicketState = {
  ticket: string;
  provider: Extract<IdentityProvider, 'evm'>;
  address: string;
};

type Props = {
  providers?: AuthConfig['providers'];
  loading: IdentityProvider | null;
  walletTicket: WalletTicketState | null;
  walletEmail: string;
  walletCode: string;
  walletCodeSent: boolean;
  onOAuth: (provider: Extract<IdentityProvider, 'github' | 'google'>) => void;
  onWallet: (
    provider: Extract<IdentityProvider, 'evm'>,
    config: WalletProviderPublicConfig
  ) => void;
  onWalletEmailChange: (value: string) => void;
  onWalletCodeChange: (value: string) => void;
  onSendWalletCode: () => void;
  onCompleteWallet: () => void;
};

export function JwtSocialSignIn(props: Props) {
  const { providers } = props;
  const hasOAuth = providers?.github.enabled || providers?.google.enabled;
  const hasWallet = providers?.evm.enabled;

  if (!hasOAuth && !hasWallet) {
    return null;
  }

  return (
    <Box sx={{ mb: 3 }}>
      <ProviderButtons {...props} />
      {props.walletTicket && <WalletEmailBinding {...props} walletTicket={props.walletTicket} />}
    </Box>
  );
}

function ProviderButtons({ providers, loading, onOAuth, onWallet }: Props) {
  const { t } = useTranslate('auth');

  return (
    <>
      <Divider sx={{ mb: 2 }}>{t('social.or')}</Divider>
      <Box sx={{ gap: 1.5, display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' } }}>
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
        {providers?.evm.enabled && (
          <ProviderButton
            label={t('social.evm')}
            icon="solar:wad-of-money-bold"
            loading={loading === 'evm'}
            onClick={() => onWallet('evm', providers.evm)}
          />
        )}
      </Box>
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

function WalletEmailBinding({
  walletTicket,
  walletEmail,
  walletCode,
  walletCodeSent,
  loading,
  onWalletEmailChange,
  onWalletCodeChange,
  onSendWalletCode,
  onCompleteWallet,
}: Props & { walletTicket: WalletTicketState }) {
  const { t } = useTranslate('auth');

  return (
    <Box sx={{ mt: 2, gap: 1.5, display: 'flex', flexDirection: 'column' }}>
      <Typography variant="body2" color="text.secondary">
        {t('social.walletEmailRequired', { address: walletTicket.address })}
      </Typography>
      <BindingText
        label={t('fields.email')}
        value={walletEmail}
        placeholder={t('placeholders.email')}
        onChange={onWalletEmailChange}
      />
      <Button
        variant="outlined"
        color="inherit"
        loading={loading === walletTicket.provider}
        onClick={onSendWalletCode}
      >
        {walletCodeSent ? t('social.resendCode') : t('actions.sendRegistrationCode')}
      </Button>
      <BindingText
        label={t('fields.emailVerificationCode')}
        value={walletCode}
        placeholder={t('placeholders.emailVerificationCode')}
        onChange={onWalletCodeChange}
      />
      <Button
        variant="contained"
        color="inherit"
        loading={loading === walletTicket.provider}
        onClick={onCompleteWallet}
      >
        {t('social.completeWallet')}
      </Button>
    </Box>
  );
}

function BindingText({
  label,
  value,
  placeholder,
  onChange,
}: {
  label: string;
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      fullWidth
      label={label}
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{ inputLabel: { shrink: true } }}
    />
  );
}
