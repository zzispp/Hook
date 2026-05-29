import type { IdentityProvider } from 'src/types/rbac';

export function providerLabel(provider: IdentityProvider) {
  const labels: Record<IdentityProvider, string> = {
    github: 'GitHub',
    google: 'Google',
    evm: 'EVM',
    solana: 'Solana',
  };
  return labels[provider];
}

export function providerColor(provider: IdentityProvider) {
  const colors: Record<IdentityProvider, 'primary' | 'info' | 'success' | 'warning'> = {
    github: 'primary',
    google: 'info',
    evm: 'warning',
    solana: 'success',
  };
  return colors[provider];
}
