# Blockchain Auth Provider Settings

## Goal

Update admin shortcut login settings so EVM and Solana are separated, each can configure network, statement, and domain source behavior. Wallet domain must use `public_base_url`; EVM and Solana cannot be enabled without `public_base_url`. EVM Chain IDs becomes a multi-select network picker with ETH selected by default and options ETH, BSC, ARB.

## Scope

- Inspect current frontend and backend configuration flow.
- Update frontend section layout and controls.
- Update frontend validation.
- Update backend validation/storage mapping if required.
- Run focused validation.

