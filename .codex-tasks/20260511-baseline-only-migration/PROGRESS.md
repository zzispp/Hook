# Progress

- 已确认当前入口仍调用 SeaORM `Migrator::up`。
- 已移除 SeaORM migration 队列入口，改为 `migration/development.rs` 直接重建当前 baseline。
- 已将 date-based baseline 目录改为 `migration/baseline`。
- 已验证 `cargo run -p backend -- migration up` 会重建当前 baseline，`migration status` 显示 18/18 张表存在。
