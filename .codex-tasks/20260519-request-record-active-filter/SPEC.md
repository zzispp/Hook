# Request Record Active Filter Fix

## Goal

Fix request record admin filtering so users can inspect in-flight requests without missing pending records, and so terminal records do not linger in active/transferring filtered views after active polling observes completion.

## Scope

- Compare `/Users/bubu/ZwjProjects/aether` request-record implementation.
- Keep Hook status semantics explicit: raw status filters stay exact; active/in-progress is a separate semantic filter if implemented.
- Update backend/frontend/i18n seed files only where required.
- Validate with available lint/build/test commands.

## Non-goals

- No compatibility fallback or mock success path.
- No unrelated refactor.
- Do not touch existing unrelated user worktree changes.
