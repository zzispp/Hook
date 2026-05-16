# 进度记录

## 2026-05-15

- 已确认当前工作区干净。
- 已确认仓库没有已落地的 `performance_monitoring` 模块。
- 已读取系统设置、迁移、storage、request record cleanup、菜单权限、前端路由与设置页面关键路径。
- 策略：以快照表为性能边界，worker 异步聚合，API 查询只读快照，`all` 强制使用 day 或安全降采样粒度。
- 已完成系统设置字段链路，`cargo test -p setting validation -- --nocapture` 通过；本机无 GNU `timeout`，用 Perl alarm 执行限时校验。
- 已完成快照 store、表结构、索引、`ALL`/30d 粒度策略与 storage 集成测试。`ALL` 查询 SQL 已断言不包含 `request_records`。
- 已完成性能监控 worker、API 路由、管理员菜单和 API 权限绑定。backend `performance_monitoring` 与 `defaults` 目标测试通过。
- 已完成前端性能监控页面、`ALL` tab、系统设置清理字段与 i18n seed；`pnpm lint:frontend` 通过。
- 已修正 `ALL` 查询策略：只读 `performance_monitoring_snapshots` 的 day 桶，并用数据库窗口函数等距降采样到 `MAX_SERIES_POINTS` 内，避免保留期超过 720 天时只展示最早 720 个桶。
- 已完成最终验证：`cargo fmt --check`、`cargo check -p backend`、`perl -e 'alarm 60; exec @ARGV' cargo test --workspace`、`pnpm lint:frontend` 均通过；本机 `just` 不在 PATH，因此使用 justfile 等价命令执行。
