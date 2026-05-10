# Baseline Only Migration

开发期移除迁移链，保留一个当前 baseline 执行入口。`migration up` 不再依赖历史 migration 是否已应用，而是确保当前 schema/seed 一步到位。
