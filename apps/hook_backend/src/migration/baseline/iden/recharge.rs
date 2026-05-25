use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum RechargePackages {
    Table,
    Id,
    Name,
    Description,
    RechargeAmount,
    GiftAmount,
    Status,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum RechargeOrders {
    Table,
    Id,
    OrderNo,
    UserId,
    PackageId,
    PackageName,
    RechargeAmount,
    GiftAmount,
    TotalArrivalAmount,
    PayableAmount,
    Status,
    PaymentChannelCode,
    PaymentChannelName,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum PaymentChannels {
    Table,
    Code,
    Name,
    Enabled,
    RegisteredAt,
    UpdatedAt,
}
