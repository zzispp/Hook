# Progress

## Recovery

- 任务: 修复系统管理员生成卡密外键失败
- 形态: single-full
- 进度: 3/3
- 当前: 完成
- 文件: .codex-tasks/20260514-card-code-generator-fk/TODO.csv

## Log

- 2026-05-14: 定位到 `admin_generate_codes` 将 `CurrentUser.id` 写入 `created_by_user_id`；配置系统管理员来自 `ConfigSystemUserProvider`，是虚拟用户，不保证存在于 `users` 表，因此触发 `card_codes_created_by_user_id_fkey`。
- 2026-05-14: `operator_user_id` 已改为系统用户返回 `None`、数据库用户保留 id；`cargo test -q -p card_code` 通过。
- 2026-05-14: 验证通过：`cargo check -q`、`pnpm lint:frontend`、`git diff --check`。
