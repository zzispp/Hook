# Progress

## Recovery

- 任务: Fix backend startup schema push failure.
- 形态: single-full
- 进度: 1/3
- 当前: Add explicit schema push configuration.
- 文件: `.codex-tasks/20260507-backend-schema-startup/TODO.csv`
- 下一步: Update config, storage connect options, and backend main.

## Log

- Context7 Toasty docs: `push_schema` is for quick schema setup/prototyping; migration system is separate.
- Brave search and local Toasty 0.4 source confirm PostgreSQL `push_schema` iterates tables and calls `create_table`; it is not `IF NOT EXISTS`.
- Decision: normal backend startup should not unconditionally mutate schema; make schema push explicit config.
