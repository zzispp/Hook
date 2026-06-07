# Non-stream Terminal Guard

## Goal
Fix non-stream proxy attempts being overwritten or surfaced as `cancelled / 499` when the non-stream attempt has already reached an explicit terminal success or failure path.

## Scope
- Inspect existing `codex/fix-nonstream-terminal-guard` worktree changes.
- Keep stream handoff semantics intact.
- Ensure non-stream response read/conversion/status failures write their real terminal record instead of a cancellation record.
- Validate with focused backend tests where feasible.

## Out of Scope
- No silent fallback or mock success paths.
- No broad lifecycle rewrite beyond the non-stream terminal ownership boundary.
