# 2026-05-14 Billing And Access Real Flow

## Goal

Run a real end-to-end Hook validation for user status limits, API-token status limits, token balance limits, user wallet balance limits, billing-group multipliers, wallet/token charge accounting, New API-compatible insufficient-balance responses, provider retry/timeout, equal-priority random selection, and cache affinity.

## Scope

- Inspect current backend implementation before asserting behavior.
- Reuse the existing real-flow backend, database, Redis, and request helpers where practical.
- Seed dedicated local database fixtures for users, API tokens, wallets, providers, routes, model bindings, billing groups, and menu prerequisites.
- Use upstream provider keys from environment variables or the current operator shell only; do not write provider secrets into source files or task artifacts.
- Exercise real proxy requests against the local backend and real upstream providers.
- Compare insufficient-balance response status and shape against the local `new-api` repository behavior when discoverable.
- Write a runnable real validation script with explicit failures and JSON evidence.

## Constraints

- No mocked success paths and no silent fallback behavior.
- Do not hardcode upstream provider secrets or generated bearer tokens in committed files.
- Let real upstream, DB, Redis, and backend failures surface as explicit errors.
- Preserve unrelated repository changes.
