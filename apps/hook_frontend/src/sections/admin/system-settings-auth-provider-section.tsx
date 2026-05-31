'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';

const OAUTH_CALLBACK_PATH_PREFIX = '/auth/oauth/callback';
const EVM_NETWORK_OPTIONS = [
  { label: 'ETH', value: '1' },
  { label: 'BSC', value: '56' },
  { label: 'ARB', value: '42161' },
];

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function AuthProviderFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <Typography variant="subtitle2">{t('systemSettings.authProviders.title')}</Typography>
      <OAuthProviderFields name="github" form={form} setForm={setForm} />
      <OAuthProviderFields name="google" form={form} setForm={setForm} />
      <WalletProviderFields form={form} setForm={setForm} />
    </Stack>
  );
}

function OAuthProviderFields({ name, form, setForm }: Props & { name: 'github' | 'google' }) {
  const { t } = useTranslate('admin');
  const prefix = name === 'github' ? 'auth_github' : 'auth_google';
  const enabled = name === 'github' ? form.auth_github_enabled : form.auth_google_enabled;
  const clientId = name === 'github' ? form.auth_github_client_id : form.auth_google_client_id;
  const secret =
    name === 'github' ? form.auth_github_client_secret : form.auth_google_client_secret;
  const secretSet =
    name === 'github' ? form.auth_github_client_secret_set : form.auth_google_client_secret_set;
  const callbackAddress = oauthCallbackAddress(
    form.public_base_url,
    name,
    t('systemSettings.authProviders.publicBaseUrlMissing')
  );

  return (
    <Stack spacing={2}>
      <SwitchRow
        checked={enabled}
        disabled={providerSwitchDisabled(form.public_base_url, enabled)}
        helperText={providerSwitchHelper(form.public_base_url, t)}
        label={t(`systemSettings.authProviders.${name}.enabled`)}
        onChange={(checked) =>
          setForm((current) => ({ ...current, [`${prefix}_enabled`]: checked }))
        }
      />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          label={t('systemSettings.authProviders.clientId')}
          value={clientId}
          onChange={(value) =>
            setForm((current) => ({ ...current, [`${prefix}_client_id`]: value }))
          }
        />
        <TextFieldRow
          type="password"
          label={t('systemSettings.authProviders.clientSecret')}
          value={secret}
          helperText={secretSet ? t('systemSettings.authProviders.secretSet') : undefined}
          onChange={(value) =>
            setForm((current) => ({ ...current, [`${prefix}_client_secret`]: value }))
          }
        />
      </Stack>
      <CallbackAddressField value={callbackAddress} />
    </Stack>
  );
}

function CallbackAddressField({ value }: { value: string }) {
  const { t } = useTranslate('admin');

  return <InfoValueField label={t('systemSettings.authProviders.callbackAddress')} value={value} />;
}

function InfoValueField({ label, value }: { label: string; value: string }) {
  return (
    <Alert severity="info" sx={{ py: 1 }}>
      <Typography variant="caption" sx={{ color: 'text.secondary' }}>
        {label}
      </Typography>
      <Typography component="div" variant="body2" sx={{ mt: 0.5, fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Alert>
  );
}

function WalletProviderFields({ form, setForm }: Props) {
  return (
    <Stack spacing={2}>
      <EvmWalletFields form={form} setForm={setForm} />
    </Stack>
  );
}

function oauthCallbackAddress(
  publicBaseUrl: string,
  provider: 'github' | 'google',
  missingBase: string
) {
  const baseUrl = publicBaseUrl.trim().replace(/\/+$/, '');
  if (!baseUrl) {
    return missingBase;
  }

  return `${baseUrl}${OAUTH_CALLBACK_PATH_PREFIX}/${provider}`;
}

function EvmWalletFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');
  const selectedNetworks = evmNetworkValues(form.auth_evm_chain_ids);

  return (
    <Stack spacing={2} sx={walletProviderSx}>
      <Typography variant="subtitle2">{t('systemSettings.authProviders.evm.title')}</Typography>
      <SwitchRow
        checked={form.auth_evm_enabled}
        disabled={providerSwitchDisabled(form.public_base_url, form.auth_evm_enabled)}
        helperText={providerSwitchHelper(form.public_base_url, t)}
        label={t('systemSettings.authProviders.evm.enabled')}
        onChange={(checked) => setForm((current) => ({ ...current, auth_evm_enabled: checked }))}
      />
      <TextFieldRow
        select
        label={t('systemSettings.authProviders.evm.networks')}
        value={selectedNetworks}
        SelectProps={{
          multiple: true,
          renderValue: evmNetworkRenderValue,
        }}
        onChange={(value) =>
          setForm((current) => ({
            ...current,
            auth_evm_chain_ids: evmNetworkValues(value).join(','),
          }))
        }
      >
        {EVM_NETWORK_OPTIONS.map((option) => (
          <MenuItem key={option.value} value={option.value}>
            <Checkbox checked={selectedNetworks.includes(option.value)} />
            <ListItemText primary={option.label} secondary={`Chain ID ${option.value}`} />
          </MenuItem>
        ))}
      </TextFieldRow>
      <TextFieldRow
        label={t('systemSettings.authProviders.evm.statement')}
        value={form.auth_evm_statement}
        onChange={(value) => setForm((current) => ({ ...current, auth_evm_statement: value }))}
      />
      <WalletDomainField publicBaseUrl={form.public_base_url} />
    </Stack>
  );
}

const walletProviderSx = { pt: 2, borderTop: 1, borderColor: 'divider' };

function WalletDomainField({ publicBaseUrl }: { publicBaseUrl: string }) {
  const { t } = useTranslate('admin');
  const domain = walletDomainValue(
    publicBaseUrl,
    t('systemSettings.authProviders.publicBaseUrlMissing')
  );

  return <InfoValueField label={t('systemSettings.authProviders.walletDomain')} value={domain} />;
}

function walletDomainValue(publicBaseUrl: string, missingBase: string) {
  const value = publicBaseUrl.trim();
  if (!value) {
    return missingBase;
  }
  try {
    return new URL(value).host;
  } catch {
    return value.replace(/\/+$/, '');
  }
}

function providerSwitchDisabled(publicBaseUrl: string, enabled: boolean) {
  return !enabled && !publicBaseUrl.trim();
}

function providerSwitchHelper(publicBaseUrl: string, t: ReturnType<typeof useTranslate>['t']) {
  return publicBaseUrl.trim() ? undefined : t('systemSettings.authProviders.publicBaseUrlMissing');
}

function evmNetworkValues(value: string) {
  const supportedValues = new Set(EVM_NETWORK_OPTIONS.map((option) => option.value));
  return value
    .split(',')
    .map((item) => item.trim())
    .filter((item) => supportedValues.has(item));
}

function evmNetworkRenderValue(selected: unknown) {
  const values = Array.isArray(selected) ? selected.map(String) : [];
  return values.map(evmNetworkLabel).filter(Boolean).join(', ');
}

function evmNetworkLabel(value: string) {
  return EVM_NETWORK_OPTIONS.find((option) => option.value === value)?.label ?? '';
}
