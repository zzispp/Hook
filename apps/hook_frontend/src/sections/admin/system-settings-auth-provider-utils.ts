import type { SystemSettingsForm } from './system-settings-utils';
import type { SystemSettings, SystemSettingsUpdate } from 'src/types/system-setting';

export const DEFAULT_AUTH_PROVIDER_FORM = {
  auth_github_enabled: false,
  auth_github_client_id: '',
  auth_github_client_secret: '',
  auth_github_client_secret_set: false,
  auth_google_enabled: false,
  auth_google_client_id: '',
  auth_google_client_secret: '',
  auth_google_client_secret_set: false,
  auth_evm_enabled: false,
  auth_evm_chain_ids: '1',
  auth_solana_enabled: false,
  auth_solana_network: 'mainnet-beta',
  auth_wallet_domain: '',
  auth_wallet_statement: 'Sign in to Hook',
};

export function authProviderFormFromSettings(settings: SystemSettings) {
  return {
    auth_github_enabled: settings.auth_github_enabled,
    auth_github_client_id: settings.auth_github_client_id,
    auth_github_client_secret: '',
    auth_github_client_secret_set: settings.auth_github_client_secret_set,
    auth_google_enabled: settings.auth_google_enabled,
    auth_google_client_id: settings.auth_google_client_id,
    auth_google_client_secret: '',
    auth_google_client_secret_set: settings.auth_google_client_secret_set,
    auth_evm_enabled: settings.auth_evm_enabled,
    auth_evm_chain_ids: settings.auth_evm_chain_ids,
    auth_solana_enabled: settings.auth_solana_enabled,
    auth_solana_network: settings.auth_solana_network,
    auth_wallet_domain: settings.auth_wallet_domain,
    auth_wallet_statement: settings.auth_wallet_statement,
  };
}

export function authProviderPayloadFields(form: SystemSettingsForm) {
  return {
    auth_github_enabled: form.auth_github_enabled,
    auth_github_client_id: form.auth_github_client_id,
    auth_google_enabled: form.auth_google_enabled,
    auth_google_client_id: form.auth_google_client_id,
    auth_evm_enabled: form.auth_evm_enabled,
    auth_evm_chain_ids: form.auth_evm_chain_ids,
    auth_solana_enabled: form.auth_solana_enabled,
    auth_solana_network: form.auth_solana_network,
    auth_wallet_domain: form.auth_wallet_domain,
    auth_wallet_statement: form.auth_wallet_statement,
  };
}

export function applyAuthProviderSecretPayload(
  payload: SystemSettingsUpdate,
  form: SystemSettingsForm
) {
  if (form.auth_github_client_secret.trim()) {
    payload.auth_github_client_secret = form.auth_github_client_secret.trim();
  }
  if (form.auth_google_client_secret.trim()) {
    payload.auth_google_client_secret = form.auth_google_client_secret.trim();
  }
}
