# Storage Scheduler Clean Build

## Goal

让 `cargo check -p storage` 不再输出 scheduler SeaORM entity 的 dead code warning，同时不删除实际数据库访问所需的 ORM 映射。

## Scope

- Inspect scheduler storage entity usage.
- Remove truly unused code if found.
- If entity code is required, adjust module reachability consistently with existing storage patterns.
- Validate Rust build is warning-clean for storage.

## Out Of Scope

- Changing scheduler runtime behavior.
- Changing database schema.
