# 08 Validation

Run full validation after all child tasks complete.

Acceptance:
- `just test` passes.
- `pnpm --filter hook_frontend lint` passes.
- Read-only local DB verification shows multi-endpoint multi-key requests no longer create endpoint x key x retry dot explosions.
