'use client';

import type { PropsWithChildren } from 'react';

import { useState } from 'react';
import { http, WagmiProvider } from 'wagmi';
import { bsc, mainnet, arbitrum } from 'wagmi/chains';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { getDefaultConfig, RainbowKitProvider } from '@rainbow-me/rainbowkit';

const WALLET_CONNECT_PROJECT_ID = '38e45fe486a77677b8522f7f182e242c';

const walletConnectorConfig = getDefaultConfig({
  appName: 'Hook',
  projectId: WALLET_CONNECT_PROJECT_ID,
  chains: [mainnet, bsc, arbitrum],
  transports: {
    [mainnet.id]: http(),
    [bsc.id]: http(),
    [arbitrum.id]: http(),
  },
  ssr: true,
});

export function WalletConnectorProvider({ children }: PropsWithChildren) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <WagmiProvider config={walletConnectorConfig}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider>{children}</RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
