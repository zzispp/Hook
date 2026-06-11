# Quick Import Sync Event Dedupe

## Goal
Diagnose and fix duplicate administrator notifications when a NewAPI quick import token changes upstream group while the configured action is not `sync`.

## Boundary
- Inspect local Postgres state for the reported provider/key.
- Fix event generation so "group synced" is emitted only when the new upstream group is actually accepted.
- Add regression coverage for disable/report behavior.
- Run targeted Rust validation.
