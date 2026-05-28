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
    PaymentMethod,
    ProviderTradeNo,
    PaymentRequestJson,
    RefundStatus,
    RefundAmount,
    PaidAt,
    RefundedAt,
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
    ConfigJson,
    EncryptedSecret,
    RegisteredAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum PaymentCallbackRecords {
    Table,
    Id,
    PaymentChannelCode,
    CallbackKind,
    HttpMethod,
    OrderNo,
    ProviderTradeNo,
    PaymentMethod,
    TradeStatus,
    Status,
    Settled,
    ErrorMessage,
    RawParamsJson,
    ReceivedAt,
    ProcessedAt,
}
