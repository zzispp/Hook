# Progress

## Recovery

- 任务: 卡密列表复制、类型省略和 IP 记录修复
- 形态: single-full
- 进度: 3/3
- 当前: 完成
- 文件: .codex-tasks/20260514-card-code-row-polish/TODO.csv

## Log

- 2026-05-14: 用户确认卡密状态就是单一状态字段，未使用时显示启用/停用，使用后显示已使用；本次不新增使用状态列。
- 2026-05-14: 已增加直连 IP fallback、卡密复制按钮、类型列单行省略；补充生成金额映射单测确认 gift 类型写入赠款余额。`cargo test -q -p card_code`、`cargo check -q`、`pnpm lint:frontend`、`git diff --check` 通过。
- 2026-05-14: `pnpm build:frontend` 通过，构建期间仍输出既有 `Axios error: unauthorized` 但退出码为 0；`cargo fmt` 后 `cargo check -q` 继续通过。
