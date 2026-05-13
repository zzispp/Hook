# Progress

## Recovery

任务: Fully fix scheduler candidate and trace dot explosion.
形态: epic
进度: 8/8
当前: All child tasks completed.
文件: .codex-tasks/20260513-candidate-routing-epic/SUBTASKS.csv
下一步: None.

## Log

- Created Epic for the remaining scheduler model work.
- Scope includes multi-key route compaction, endpoint conversion fallback, terminal audit cleanup, hierarchical trace UI, explicit candidate budget semantics, and end-to-end validation.
- Implemented provider-level candidate routes so initial audit rows are per provider route, not endpoint x key x retry.
- Route materialization now selects the concrete endpoint/key at attempt time; retry budget covers every endpoint/key route option at least once and then cycles within the explicit retry budget.
- Terminal cleanup marks remaining available rows as unused on success, terminal upstream failure, no-response failure, and websocket connection exhaustion.
- Trace UI groups by provider and shows real attempts with a collapsed +N summary for hidden unscheduled rows.
- Validation passed: `cargo test -p backend candidate -- --nocapture`, `cargo test -p backend matching_candidate_parts -- --nocapture`, `just test`, and `pnpm --filter hook_frontend lint`.
- DB verification passed on local Postgres: new request `019e1f1c-97e4-73e2-b493-4971b0a196f2` wrote 1 candidate row with `candidate_index=0,retry_index=0,status=success`; old records in the same DB show the previous 9-row pattern with 3 candidate indexes.
