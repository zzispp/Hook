# Progress

## Recovery

任务: billing/access/routing 真实流验证。
形态: single-full
进度: 6/6
当前: Completed.
文件: `.codex-tasks/20260514-billing-access-real-flow/TODO.csv`
下一步: Inspect backend code, DB schema, existing real-flow helpers, and local New API behavior.

## Log

- 2026-05-14: 初始化任务。目标是把用户禁用、令牌禁用、令牌余额限制、用户钱包余额限制、计费分组倍率、钱包/令牌扣费、New API 兼容余额错误、provider 重试/超时、同优先级随机选择与缓存亲和全部放进真实 DB/API 验证。
- 2026-05-14: 已补齐真实验证脚本，拆分为环境、fixture、DB 控制、代理客户端、断言、访问/计费场景、路由场景和运行时模块。入口与所有 helper 均通过 `node --check`。
- 2026-05-14: 真实脚本连接本地 DB、启动本地后端、调用 Hook 与 Ekan8 上游完成验证；结果写入 `raw/results.json`，结论写入 `raw/final-evidence.md`。发现禁用用户、令牌额度、钱包额度、钱包扣费、cache_affinity 冷启动随机选择未满足预期；禁用 token、倍率计费、令牌 used_quota、retry、timeout、load_balance、cache_affinity warm hit、Ekan8 映射请求通过。
- 2026-05-14: 后端修复后复跑真实脚本，14/14 场景通过；`raw/results.json` 和 `raw/final-evidence.md` 已更新为修复后的真实证据。
