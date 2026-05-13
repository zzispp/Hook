use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    Email,
    Role,
    IsActive,
    IsDeleted,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
    AuthSource,
    EmailVerified,
    RateLimitRpm,
    QuotaMode,
}
