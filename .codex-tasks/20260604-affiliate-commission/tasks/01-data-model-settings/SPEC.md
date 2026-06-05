# 建立返佣数据模型和系统设置字段

## Goal

为邀请返佣功能提供稳定的数据结构：用户邀请码字段、邀请关系字段、返佣记录表、系统返佣比例设置。

## Acceptance

- Baseline migration 能创建新增列和返佣表。
- SeaORM entities、storage record/type、domain types 都能表达新增字段。
- 系统设置更新、响应、校验链路包含 `affiliate_commission_percent`。
- `cargo check -p types -p storage -p setting` 通过。
