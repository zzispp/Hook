# Billing Access Fixes

## Goal

Fix the real failures found by `.codex-tasks/20260514-billing-access-real-flow`:

- reject disabled users before upstream proxy execution;
- reject exhausted API tokens before upstream proxy execution;
- reject exhausted finite wallets before upstream proxy execution;
- settle successful LLM billing into the user wallet ledger;
- make cold `cache_affinity` equal-priority routing randomize before a cache entry exists;
- rerun static checks, targeted tests, and the real DB/upstream validation script.

## Constraints

- No mock success paths or silent fallbacks.
- Failures must surface as explicit proxy errors.
- Wallet charging must be transactional with request success recording as much as current storage boundaries allow.
- Keep implementation within repository code-size and complexity limits.
- Do not write provider secrets into source or artifacts.
