'use client';

import type { MutableRefObject } from 'react';
import type { IdentityProvider } from 'src/types/rbac';

import { useRef, useEffect, useCallback } from 'react';
import { useConnectModal } from '@rainbow-me/rainbowkit';
import { useAccount, useSignMessage, useSwitchChain } from 'wagmi';

export type WalletSignatureResult = {
  provider: Extract<IdentityProvider, 'evm'>;
  address: string;
  signature: string;
  chainId?: number;
};

export type WalletAccountResult = Omit<WalletSignatureResult, 'signature'>;

type WalletConnectRequest = {
  provider: Extract<IdentityProvider, 'evm'>;
  chainId?: number;
};

type PendingConnect = {
  request: WalletConnectRequest;
  modalSeen: boolean;
  resolve: (account: WalletAccountResult) => void;
  reject: (error: Error) => void;
};

type SwitchChainAsync = ReturnType<typeof useSwitchChain>['switchChainAsync'];

export function useWalletSigning() {
  const pendingConnect = useRef<PendingConnect | null>(null);
  const { address, chainId: activeChainId, isConnected } = useAccount();
  const { openConnectModal, connectModalOpen } = useConnectModal();
  const { switchChainAsync } = useSwitchChain();
  const { signMessageAsync } = useSignMessage();

  const connectWalletAccount = useCallback(
    async (request: WalletConnectRequest): Promise<WalletAccountResult> => {
      if (isConnected && address) {
        return ensureWalletAccount({ request, address, activeChainId, switchChainAsync });
      }
      if (!openConnectModal) {
        throw new Error('EVM wallet connector is not available');
      }
      return waitForRainbowKitConnection({ request, openConnectModal, pendingConnect });
    },
    [activeChainId, address, isConnected, openConnectModal, switchChainAsync]
  );

  const signWalletMessage = useCallback(
    async (input: WalletAccountResult & { message: string }): Promise<WalletSignatureResult> => {
      const account = await ensureWalletAccount({
        request: input,
        address,
        activeChainId,
        switchChainAsync,
      });
      assertMatchingAddress(account.address, input.address);
      const signature = await signMessageAsync({ message: input.message });
      return { ...account, signature };
    },
    [activeChainId, address, signMessageAsync, switchChainAsync]
  );

  usePendingRainbowKitConnection({
    pendingConnect,
    address,
    activeChainId,
    isConnected,
    connectModalOpen,
    switchChainAsync,
  });

  return { connectWalletAccount, signWalletMessage };
}

function waitForRainbowKitConnection({
  request,
  openConnectModal,
  pendingConnect,
}: {
  request: WalletConnectRequest;
  openConnectModal: () => void;
  pendingConnect: MutableRefObject<PendingConnect | null>;
}) {
  return new Promise<WalletAccountResult>((resolve, reject) => {
    pendingConnect.current = { request, resolve, reject, modalSeen: false };
    openConnectModal();
  });
}

function usePendingRainbowKitConnection({
  pendingConnect,
  address,
  activeChainId,
  isConnected,
  connectModalOpen,
  switchChainAsync,
}: {
  pendingConnect: MutableRefObject<PendingConnect | null>;
  address?: string;
  activeChainId?: number;
  isConnected: boolean;
  connectModalOpen: boolean;
  switchChainAsync: SwitchChainAsync;
}) {
  useEffect(() => {
    const pending = pendingConnect.current;
    if (!pending) return;
    if (isConnected && address) {
      resolvePendingConnection({ pendingConnect, pending, address, activeChainId, switchChainAsync });
      return;
    }
    if (!connectModalOpen && pending.modalSeen) {
      pendingConnect.current = null;
      pending.reject(new Error('EVM wallet connection was cancelled'));
      return;
    }
    if (connectModalOpen && !pending.modalSeen) {
      pendingConnect.current = { ...pending, modalSeen: true };
    }
  }, [activeChainId, address, connectModalOpen, isConnected, pendingConnect, switchChainAsync]);
}

function resolvePendingConnection({
  pendingConnect,
  pending,
  address,
  activeChainId,
  switchChainAsync,
}: {
  pendingConnect: MutableRefObject<PendingConnect | null>;
  pending: PendingConnect;
  address: string;
  activeChainId?: number;
  switchChainAsync: SwitchChainAsync;
}) {
  pendingConnect.current = null;
  ensureWalletAccount({ request: pending.request, address, activeChainId, switchChainAsync })
    .then(pending.resolve)
    .catch(pending.reject);
}

async function ensureWalletAccount({
  request,
  address,
  activeChainId,
  switchChainAsync,
}: {
  request: WalletConnectRequest;
  address?: string;
  activeChainId?: number;
  switchChainAsync: SwitchChainAsync;
}): Promise<WalletAccountResult> {
  if (!address) {
    throw new Error('EVM wallet account not found');
  }
  const chainId = await ensureWalletChain({ request, activeChainId, switchChainAsync });
  return { provider: request.provider, address, chainId };
}

async function ensureWalletChain({
  request,
  activeChainId,
  switchChainAsync,
}: {
  request: WalletConnectRequest;
  activeChainId?: number;
  switchChainAsync: SwitchChainAsync;
}) {
  if (!request.chainId || activeChainId === request.chainId) {
    return request.chainId ?? activeChainId;
  }
  const chain = await switchChainAsync({ chainId: request.chainId });
  return chain.id;
}

function assertMatchingAddress(actual: string, expected: string) {
  if (actual.toLowerCase() !== expected.toLowerCase()) {
    throw new Error('Connected EVM account changed before signing');
  }
}
