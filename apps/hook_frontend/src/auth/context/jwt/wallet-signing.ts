import type { IdentityProvider } from 'src/types/rbac';

type EthereumProvider = {
  request: (args: { method: string; params?: unknown[] }) => Promise<unknown>;
};

type SolanaProvider = {
  connect: () => Promise<{ publicKey?: { toString: () => string } }>;
  signMessage: (message: Uint8Array, encoding?: string) => Promise<{ signature: Uint8Array }>;
};

export type WalletSignatureResult = {
  provider: Extract<IdentityProvider, 'evm' | 'solana'>;
  address: string;
  signature: string;
  chainId?: number;
  network?: string;
};

export type WalletAccountResult = Omit<WalletSignatureResult, 'signature'>;

const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

export async function connectWalletAccount({
  provider,
  chainId,
  network,
}: {
  provider: Extract<IdentityProvider, 'evm' | 'solana'>;
  chainId?: number;
  network?: string;
}): Promise<WalletAccountResult> {
  if (provider === 'evm') {
    const address = await connectEvmAddress();
    return { provider, address, chainId };
  }
  const address = await connectSolanaAddress();
  return { provider, address, network };
}

export async function signWalletMessage({
  provider,
  message,
  chainId,
  network,
}: {
  provider: Extract<IdentityProvider, 'evm' | 'solana'>;
  message: string;
  chainId?: number;
  network?: string;
}): Promise<WalletSignatureResult> {
  if (provider === 'evm') {
    return signEvmMessage(message, chainId);
  }
  return signSolanaMessage(message, network);
}

async function signEvmMessage(message: string, chainId?: number): Promise<WalletSignatureResult> {
  const address = await connectEvmAddress();
  const ethereum = requireWindowProvider('ethereum') as EthereumProvider;
  const signature = (await ethereum.request({
    method: 'personal_sign',
    params: [message, address],
  })) as string;
  return { provider: 'evm', address, signature, chainId };
}

async function signSolanaMessage(
  message: string,
  network?: string
): Promise<WalletSignatureResult> {
  const address = await connectSolanaAddress();
  const solana = requireWindowProvider('solana') as SolanaProvider;
  const encodedMessage = new TextEncoder().encode(message);
  const signed = await solana.signMessage(encodedMessage, 'utf8');
  return { provider: 'solana', address, signature: base58Encode(signed.signature), network };
}

async function connectEvmAddress() {
  const ethereum = requireWindowProvider('ethereum') as EthereumProvider;
  const accounts = (await ethereum.request({ method: 'eth_requestAccounts' })) as string[];
  const address = accounts[0];
  if (!address) {
    throw new Error('EVM wallet account not found');
  }
  return address;
}

async function connectSolanaAddress() {
  const solana = requireWindowProvider('solana') as SolanaProvider;
  const connection = await solana.connect();
  const address = connection.publicKey?.toString();
  if (!address) {
    throw new Error('Solana wallet account not found');
  }
  return address;
}

function requireWindowProvider(key: 'ethereum' | 'solana') {
  const provider = windowProvider(key);
  if (!provider) {
    throw new Error(`${key === 'ethereum' ? 'EVM' : 'Solana'} wallet is not available`);
  }
  return provider;
}

function windowProvider(key: 'ethereum' | 'solana') {
  if (typeof window === 'undefined') {
    return undefined;
  }
  return (window as unknown as Record<typeof key, unknown>)[key];
}

function base58Encode(bytes: Uint8Array) {
  let zeros = 0;
  while (zeros < bytes.length && bytes[zeros] === 0) {
    zeros += 1;
  }
  const digits: number[] = [];
  for (const byte of bytes) {
    let carry = byte;
    for (let index = 0; index < digits.length; index += 1) {
      carry += digits[index] * 256;
      digits[index] = carry % 58;
      carry = Math.floor(carry / 58);
    }
    while (carry > 0) {
      digits.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }
  return '1'.repeat(zeros) + digits.reverse().map((digit) => BASE58_ALPHABET[digit]).join('');
}
