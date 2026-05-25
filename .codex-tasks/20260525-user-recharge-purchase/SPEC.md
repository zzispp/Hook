# User Recharge Purchase

## Goal

补齐用户侧充值入口和套餐购买链路：用户在钱包中心看到充值 tab，读取启用套餐，选择套餐后创建真实 pending 充值订单。

## Boundaries

- 不接入支付渠道。
- 不标记支付成功。
- 不修改钱包余额，不产生钱包流水。
- 充值关闭、套餐禁用、金额越界时明确失败。

## Validation

- 后端定向测试覆盖用户创建订单的关键规则。
- 前端 lint/build 通过。

