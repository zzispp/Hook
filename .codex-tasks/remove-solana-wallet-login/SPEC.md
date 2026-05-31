# Remove Solana Wallet Login

## Goal

Only EVM wallet login remains available and configurable. Solana-specific wallet login UI, signing, public auth config, service branches, settings fields, validation, and seed copy are removed.

## Boundaries

- Keep non-wallet historical identity/provider display code unless it blocks compilation.
- Keep EVM wallet login behavior intact.
- Do not add compatibility fallbacks or mock behavior.
