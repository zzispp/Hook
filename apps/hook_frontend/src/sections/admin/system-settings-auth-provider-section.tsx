'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';

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
  const secret = name === 'github' ? form.auth_github_client_secret : form.auth_google_client_secret;
  const secretSet = name === 'github' ? form.auth_github_client_secret_set : form.auth_google_client_secret_set;

  return (
    <Stack spacing={2}>
      <SwitchRow
        checked={enabled}
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
    </Stack>
  );
}

function WalletProviderFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SwitchRow
          checked={form.auth_evm_enabled}
          label={t('systemSettings.authProviders.evm.enabled')}
          onChange={(checked) => setForm((current) => ({ ...current, auth_evm_enabled: checked }))}
        />
        <SwitchRow
          checked={form.auth_solana_enabled}
          label={t('systemSettings.authProviders.solana.enabled')}
          onChange={(checked) => setForm((current) => ({ ...current, auth_solana_enabled: checked }))}
        />
      </Stack>
      <WalletTextFields form={form} setForm={setForm} />
    </Stack>
  );
}

function WalletTextFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          label={t('systemSettings.authProviders.evmChainIds')}
          value={form.auth_evm_chain_ids}
          placeholder="1, 137"
          onChange={(value) => setForm((current) => ({ ...current, auth_evm_chain_ids: value }))}
        />
        <TextFieldRow
          label={t('systemSettings.authProviders.solanaNetwork')}
          value={form.auth_solana_network}
          placeholder="mainnet-beta"
          onChange={(value) => setForm((current) => ({ ...current, auth_solana_network: value }))}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          label={t('systemSettings.authProviders.walletDomain')}
          value={form.auth_wallet_domain}
          placeholder="example.com"
          onChange={(value) => setForm((current) => ({ ...current, auth_wallet_domain: value }))}
        />
        <TextFieldRow
          label={t('systemSettings.authProviders.walletStatement')}
          value={form.auth_wallet_statement}
          onChange={(value) => setForm((current) => ({ ...current, auth_wallet_statement: value }))}
        />
      </Stack>
    </>
  );
}
