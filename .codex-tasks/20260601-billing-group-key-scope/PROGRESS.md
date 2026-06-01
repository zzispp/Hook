# Progress

## Log

- Started read-only inspection. Existing billing group restriction is provider-only via allowed_provider_ids.

## Recovery

任务: billing group provider key scope
形态: single-full
进度: 0/4
当前: write failing scheduler test
文件: .codex-tasks/20260601-billing-group-key-scope/TODO.csv
下一步: inspect scheduler tests and add key-scope failing case.
- Scheduler failing test was added and now passes after adding group_allowed_provider_key_ids filtering.
- Backend data flow compiles after adding billing_group_provider_keys and allowed_provider_key_ids.
- Frontend billing group form/detail/table now handles allowed_provider_key_ids and lint passes.
- Added backend provider-key ownership validation for create/update and reran cargo fmt, cargo check, group test, focused proxy scheduler test, and frontend lint.
