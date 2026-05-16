# 进度记录

## 2026-05-16

- 已确认旧磁盘空间展示把 `sysinfo` 返回的所有挂载卷直接相加；在 macOS APFS 共享容器下会重复计算，导致总量接近翻倍。
- 已新增 `performance_monitoring_disk`，磁盘容量改为当前 backend 工作目录所在文件系统的 `available / total`，不再跨所有挂载卷求和。
- 已把主机资源文案改清楚：`CPU 使用率 / 1分钟负载`、`进程 RSS / 系统已用内存`、`文件系统可用 / 总量`。
- 已把采集状态从原始枚举 `ready/unsupported` 改为 i18n 文案：`采集正常 / 不支持采集`。
- 验证通过：`cargo fmt --check`、`cargo check -p backend`、`cargo test -p backend performance_monitoring -- --nocapture`、`cargo test -p storage performance_monitoring -- --nocapture`、`pnpm lint:frontend`。
