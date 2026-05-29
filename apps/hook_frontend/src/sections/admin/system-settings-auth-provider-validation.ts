import type { SystemSettingsForm } from './system-settings-utils';

const MAX_AUTH_CLIENT_ID_LENGTH = 255;
const MAX_AUTH_CLIENT_SECRET_LENGTH = 2048;
const MAX_AUTH_WALLET_DOMAIN_LENGTH = 255;
const MAX_AUTH_WALLET_STATEMENT_LENGTH = 200;

type T = (key: string, options?: Record<string, unknown>) => string;

export function validateAuthProviderFields(form: SystemSettingsForm, t: T) {
  return (
    validateOAuthProvider(
      form.auth_github_enabled,
      form.auth_github_client_id,
      form.auth_github_client_secret,
      form.auth_github_client_secret_set,
      t
    ) ||
    validateOAuthProvider(
      form.auth_google_enabled,
      form.auth_google_client_id,
      form.auth_google_client_secret,
      form.auth_google_client_secret_set,
      t
    ) ||
    validateWalletProvider(form, t)
  );
}

function validateOAuthProvider(
  enabled: boolean,
  clientId: string,
  clientSecret: string,
  secretSet: boolean,
  t: T
) {
  if (clientId.trim().length > MAX_AUTH_CLIENT_ID_LENGTH) {
    return t('systemSettings.validation.authClientIdLength', { max: MAX_AUTH_CLIENT_ID_LENGTH });
  }
  if (clientSecret.trim().length > MAX_AUTH_CLIENT_SECRET_LENGTH) {
    return t('systemSettings.validation.authClientSecretLength', {
      max: MAX_AUTH_CLIENT_SECRET_LENGTH,
    });
  }
  if (enabled && (!clientId.trim() || (!clientSecret.trim() && !secretSet))) {
    return t('systemSettings.validation.oauthProviderIncomplete');
  }
  return '';
}

function validateWalletProvider(form: SystemSettingsForm, t: T) {
  if (form.auth_wallet_domain.trim().length > MAX_AUTH_WALLET_DOMAIN_LENGTH) {
    return t('systemSettings.validation.walletDomainLength', {
      max: MAX_AUTH_WALLET_DOMAIN_LENGTH,
    });
  }
  if (form.auth_wallet_statement.trim().length > MAX_AUTH_WALLET_STATEMENT_LENGTH) {
    return t('systemSettings.validation.walletStatementLength', {
      max: MAX_AUTH_WALLET_STATEMENT_LENGTH,
    });
  }
  if (form.auth_evm_enabled && evmChainIds(form.auth_evm_chain_ids).length === 0) {
    return t('systemSettings.validation.evmChainIdsRequired');
  }
  if (form.auth_solana_enabled && !form.auth_solana_network.trim()) {
    return t('systemSettings.validation.solanaNetworkRequired');
  }
  if ((form.auth_evm_enabled || form.auth_solana_enabled) && !form.auth_wallet_domain.trim()) {
    return t('systemSettings.validation.walletDomainRequired');
  }
  if ((form.auth_evm_enabled || form.auth_solana_enabled) && !form.auth_wallet_statement.trim()) {
    return t('systemSettings.validation.walletStatementRequired');
  }
  return '';
}

function evmChainIds(value: string) {
  return value
    .split(',')
    .map((item) => Number(item.trim()))
    .filter((item) => Number.isInteger(item) && item > 0);
}
