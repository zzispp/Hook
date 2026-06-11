# Provider Quick Import Sync

Implement automatic sync for quick-import providers. The scheduled task scans
quick-import metadata, refreshes newapi token/group state, updates managed costs
according to provider policy, and exposes per-key sync chips in admin UI.

## Scope

- Add persistent quick-import source/key/model metadata.
- Add provider-level sync settings and GET/PATCH admin API.
- Register a scheduler task with global integer config.
- Sync costs and statuses only; do not auto-create tokens or model bindings.
- Show source/key sync state in quick-import provider UI.

## Validation

- Targeted Rust unit/integration tests.
- `pnpm lint:frontend`
- `pnpm build:frontend`
- Backend targeted tests and `just test` when feasible.
