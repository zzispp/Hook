# Provider Header Rules Fix

## Goal

Make provider endpoint `header_rules` affect real upstream proxy requests.

## Scope

- Carry endpoint `header_rules` from DB scheduling snapshots into proxy candidates.
- Apply supported header actions before sending upstream HTTP requests.
- Verify with deterministic Rust tests and a real local proxy request against an echo upstream.

## Out Of Scope

- Provider body rewrite rules.
- Frontend UI changes.
