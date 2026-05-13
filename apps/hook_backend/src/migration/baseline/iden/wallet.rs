use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum Wallets {
    Table,
    Id,
    UserId,
    RechargeBalance,
    GiftBalance,
    Currency,
    Status,
    LimitMode,
    TotalRecharged,
    TotalConsumed,
    TotalRefunded,
    TotalAdjusted,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum WalletTransactions {
    Table,
    Id,
    WalletId,
    Category,
    ReasonCode,
    Amount,
    BalanceBefore,
    BalanceAfter,
    RechargeBalanceBefore,
    RechargeBalanceAfter,
    GiftBalanceBefore,
    GiftBalanceAfter,
    LinkType,
    LinkId,
    OperatorId,
    Description,
    CreatedAt,
}
