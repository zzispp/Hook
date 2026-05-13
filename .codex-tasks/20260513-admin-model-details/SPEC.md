# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Add a usage count column to the admin model management table.
- Add an admin model detail modal that shows model identity, status, base pricing, usage stats, provider coverage, and billing-group-adjusted pricing.
- Follow the existing frontend structure, backend-admin i18n seed pattern, and model/group pricing contracts.

## Non-Goals

- Do not add mock data, fake success paths, silent fallbacks, or compatibility shims.
- Do not redesign unrelated model, provider, or billing group workflows.
- Do not add frontend locale JSON files.

## Constraints

- Admin UI copy must come from backend-controlled `admin` namespace translations.
- Billing group display must make the effective price relationship explicit: base model pricing combines with group multiplier and currency.
- Keep files under project metric limits where touched; split only when required by existing file size or responsibility.
- Backend tests must use the repository timeout wrapper when run.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + pnpm/Next.js frontend
- **Package manager**: pnpm
- **Test framework**: Rust tests via `just test`; frontend validation via lint/build
- **Build command**: `pnpm build:frontend`
- **Existing test count**: not counted for this task

## Risk Assessment

- [x] External dependencies (APIs, services) — local code/contracts are available.
- [ ] Breaking changes to existing code — inspect affected call sites before edits.
- [x] Large file generation — not expected.
- [x] Long-running tests — use bounded commands where possible.

## Deliverables

- Updated admin model table with usage count.
- Model detail modal wired from admin model management.
- Billing-group effective pricing section in the modal.
- Backend i18n seed updates for any new admin UI keys.

## Done-When

- [ ] The table shows real `usage_count` from the model API response.
- [ ] The detail modal opens from the existing model management UI and shows details for the selected model.
- [ ] The modal includes billing groups, their multiplier, currency, and effective prices derived from model base pricing.
- [ ] New UI copy uses backend admin i18n seed keys in Chinese and English.
- [ ] Frontend lint/build or a narrower justified validation passes.

## Final Validation Command

```bash
pnpm lint:frontend
```

## Demo Flow

1. Open `/dashboard/admin/models`.
2. Confirm the table includes the usage count column.
3. Click a model detail action.
4. Confirm the modal shows base pricing, stats, provider coverage, and billing group pricing.
