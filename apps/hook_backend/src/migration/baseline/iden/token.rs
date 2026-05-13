use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ApiTokens {
    Table,
    Id,
    UserId,
    TokenType,
    Name,
    TokenValue,
    TokenHash,
    TokenPrefix,
    GroupCode,
    ExpiresAt,
    ModelAccessMode,
    AllowedModelIds,
    RateLimitRpm,
    QuotaLimit,
    UsedQuota,
    RequestCount,
    IsActive,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
}
