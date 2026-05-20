# Progress

## 2026-05-20

- Started from the user report: overriding `instructions` in provider endpoint body rules causes a browser console `JSON.parse` crash.
- Investigated the frontend rule editor and found render-time comparisons call `editableBodyRulesToApi`, which parses draft values before validation can block save.
- Split render-time change detection into a comparison helper that normalizes draft body rules without forcing strict JSON parsing.
- Ran `pnpm --filter hook_frontend lint`; it passed.
