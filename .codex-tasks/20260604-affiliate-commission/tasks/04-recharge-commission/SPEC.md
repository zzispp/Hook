# 充值返佣结算

## Goal

充值订单支付成功结算时，在同一个数据库事务内完成用户充值到账、邀请人钱包创建、返佣 gift_balance 入账、钱包流水和 affiliate_commissions 记录。

## Acceptance

- 返佣基数为 `recharge_orders.payable_amount`。
- `affiliate_commission_percent` 为 `0` 或用户无邀请人时不返佣。
- 邀请人钱包不存在时在结算事务内创建。
- 重复支付回调不重复返佣。
- 返佣任一步失败则结算失败并回滚。
