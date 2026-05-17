# Aether Chain Alignment Epic

## Goal

- Align Hook's HTTP model request chain with Aether's main-chain semantics while preserving Hook's cache, cooldown, usage flush, and billing-group performance advantages.
- Replace the legacy token-price billing path with Aether-style `FormulaEngine`, `billing_rules`, and `dimension_collectors`.
- Apply Hook billing groups after Aether base cost calculation.

## Non-Goals

- Do not implement Aether hub/proxy tunnel behavior.
- Do not implement Aether account-pool behavior.
- Do not add silent fallback, mock success, token estimation, or compatibility billing paths.

## Constraints

- Rust workspace and SeaORM patterns must follow existing Hook modules.
- Development-stage destructive changes are allowed.
- Billing failures must be explicit.
- Backend test commands must use the repository timeout wrapper or `timeout 60`.

## Risk Assessment

- Billing schema and request-record schema changes are breaking migrations.
- Formula evaluation must reject unsafe expressions and expose incomplete billing states.
- Main-chain failure classification must cooperate with existing Redis/DB provider cooldown.

## Child Deliverables

- Taskmaster tracking files.
- Billing schema, storage entities, and request billing snapshots.
- Formula engine, rule lookup, collectors, and billing service.
- Audit, wallet, usage, and request-chain integration.
- Aether-like failure classification with Hook cooldown.
- Tests and validation.

## Dependency Notes

- Billing schema must exist before storage wiring.
- Billing engine must exist before audit and wallet settlement integration.
- Failure classification can proceed after current executor behavior is isolated.

## Done-When

- [ ] Every row in `SUBTASKS.csv` is `DONE`
- [ ] `timeout 60 just test` passes or failures are explicitly recorded
- [ ] `timeout 60 just check` passes or failures are explicitly recorded
