# Progress

## Recovery

- 任务: 为编辑用户 modal 增加允许 provider/model，默认不限制，并同步 LLM proxy。
- 形态: single-full
- 进度: 4/4
- 当前: 完成
- 文件: `.codex-tasks/20260513-user-provider-model-limits/TODO.csv`
- 下一步: 无

## Log

- 2026-05-13: 初始化任务。
- 2026-05-13: 已定位 Hook 用户管理、类型、存储、baseline migration、LLM proxy 调度链路；Aether 参考语义为空表示不限制，调度时合并用户和 key 限制。
- 2026-05-13: 已为用户类型、存储、接口、前端用户编辑 modal 增加 allowed model/provider ID 字段，默认空数组表示不限制。
- 2026-05-13: 已将用户级限制接入 LLM proxy 调度快照和候选过滤，用户变更后刷新调度快照。
- 2026-05-13: 验证通过：cargo fmt；cargo check -p backend；60 秒超时包装下的 cargo test -p proxy/user/backend；pnpm lint:frontend；pnpm build:frontend。
