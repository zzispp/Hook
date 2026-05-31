# Use RainbowKit For EVM Wallet Login

## Goal

Replace the custom EVM wallet connector with RainbowKit/Wagmi and use WalletConnect project id `38e45fe486a77677b8522f7f182e242c`.

## Scope

- Add frontend wallet connector dependencies.
- Add RainbowKit/Wagmi/React Query providers to the app root.
- Keep the existing EVM sign-in API flow: connect wallet, request nonce, sign SIWE message, submit signature.
- Keep the existing supported EVM chains: ETH, BSC, ARB.

## Validation

- `pnpm lint:frontend`
- `pnpm build:frontend`
- Source search confirms the WalletConnect project id is configured in RainbowKit.
