# EVM Wallet Requires Account

## Goal

Remove the unauthenticated wallet email binding flow from sign-in. An EVM wallet can sign in only when the wallet identity is already linked to an existing user. If the wallet is not linked, the user must register or sign in first, then bind the wallet from the account profile page.

## Scope

- Backend wallet sign-in must stop issuing wallet binding tickets for unknown wallet addresses.
- Public auth routes for wallet email code and wallet completion must be removed from the sign-in path.
- Frontend sign-in must remove the email/code wallet binding UI and show an account-required message instead.
- Frontend sign-in must show a RainbowKit account control when the wallet is already connected so the user can disconnect or change wallet.
- Tests must cover the new unknown-wallet behavior.
