use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum CardCodeTypes {
    Table,
    Id,
    Name,
    BalanceType,
    Status,
    Remark,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum CardCodes {
    Table,
    Id,
    Code,
    BatchNo,
    TypeId,
    TypeName,
    RechargeAmount,
    GiftAmount,
    Currency,
    Status,
    Remark,
    ExpiresAt,
    CreatedByUserId,
    CreatedByUsername,
    CreatedIp,
    UsedByUserId,
    UsedByUsername,
    UsedIp,
    UsedAt,
    WalletId,
    WalletTransactionId,
    CreatedAt,
    UpdatedAt,
}
