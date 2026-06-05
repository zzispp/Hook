# 邀请返佣功能

## Goal

新增完整邀请返佣链路：所有新用户创建时生成唯一邀请码，注册/OAuth/钱包注册/管理员显式指定时绑定邀请关系，充值结算按系统设置给邀请人钱包 gift_balance 返佣。

## Delivery Boundary

- 后端数据模型、baseline migration、storage/types/service/API 全链路。
- 普通注册、OAuth 首次创建、钱包注册、管理员创建用户的邀请关系处理。
- 充值结算同事务返佣，邀请人钱包不存在时创建。
- 后台返佣设置、用户端邀请摘要和邀请链接展示。
- Rust 与前端基础验证通过。

## Fixed Decisions

- 返佣基数为 `recharge_orders.payable_amount`。
- 返佣进入邀请人的 `gift_balance`。
- 全局比例配置 `affiliate_commission_percent`，默认 `0`，范围 `0..=100`。
- 邀请关系只在账号首次创建时确定，管理员创建用户默认不绑定，显式填写才绑定。
- 不实现退款反冲。
